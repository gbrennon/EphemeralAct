use bollard::{
    Docker,
    models::{ContainerCreateBody, HostConfig},
    query_parameters::{
        CreateContainerOptionsBuilder, CreateImageOptionsBuilder, RemoveContainerOptions,
        StartContainerOptions,
    },
};
use futures_util::StreamExt;
use tokio::runtime::Runtime;

use super::podman_container::PodmanContainer;
use crate::core::ports::outbound::{
    Container, ContainerConfig, ContainerError, ContainerRuntime, HostInfo,
};

/// Podman-based container runtime adapter using the bollard crate.
///
/// Podman exposes a Docker-compatible API. This adapter connects via the
/// Podman socket, trying rootless first (`/run/user/$UID/podman/podman.sock`)
/// then falling back to the root socket (`/run/podman/podman.sock`).
pub struct PodmanRuntime {
    docker: Docker,
    runtime: Runtime,
}

impl PodmanRuntime {
    /// Create a new Podman runtime adapter.
    ///
    /// Probes the rootless socket first, then the root socket. Returns
    /// `ContainerError::NotAvailable` if neither is reachable.
    pub fn new() -> Result<Self, ContainerError> {
        let runtime = Runtime::new().map_err(|e| ContainerError::Internal(e.to_string()))?;

        // Try rootless socket first
        let uid = unsafe { libc::getuid() };
        let rootless_socket = format!("unix:///run/user/{}/podman/podman.sock", uid);
        let root_socket = "unix:///run/podman/podman.sock";

        let docker = Docker::connect_with_unix(&rootless_socket, 120, bollard::API_DEFAULT_VERSION)
            .or_else(|_| Docker::connect_with_unix(root_socket, 120, bollard::API_DEFAULT_VERSION))
            .map_err(|_| ContainerError::NotAvailable)?;

        Ok(Self { docker, runtime })
    }
}

impl ContainerRuntime for PodmanRuntime {
    fn pull_image(&self, image: &str, platform: Option<&str>) -> Result<(), ContainerError> {
        let mut options_builder = CreateImageOptionsBuilder::new().from_image(image);
        if let Some(p) = platform {
            options_builder = options_builder.platform(p);
        }
        let options = options_builder.build();

        self.runtime.block_on(async {
            let mut stream = self.docker.create_image(
                Some(options),
                None,
                None::<bollard::auth::DockerCredentials>,
            );

            while let Some(result) = stream.next().await {
                match result {
                    Ok(_info) => { /* progress */ }
                    Err(e) => {
                        return Err(ContainerError::ImagePullFailed(
                            image.to_string(),
                            e.to_string(),
                        ));
                    }
                }
            }
            Ok(())
        })
    }

    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn Container>, ContainerError> {
        let env_list: Vec<String> = config
            .env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        let host_config = HostConfig {
            binds: Some(config.binds.clone()),
            network_mode: config.network.clone(),
            ..Default::default()
        };

        let create_options = CreateContainerOptionsBuilder::new()
            .name(config.name.as_deref().unwrap_or(""))
            .platform(config.platform.as_deref().unwrap_or(""))
            .build();

        let container_config = ContainerCreateBody {
            image: Some(config.image.clone()),
            env: Some(env_list),
            cmd: config.cmd.clone(),
            entrypoint: config.entrypoint.clone(),
            working_dir: config.workdir.clone(),
            host_config: Some(host_config),
            ..Default::default()
        };

        let container = self.runtime.block_on(async {
            self.docker
                .create_container(Some(create_options), container_config)
                .await
                .map_err(|e| {
                    ContainerError::CreationFailed(
                        config.name.clone().unwrap_or_default(),
                        e.to_string(),
                    )
                })
        })?;

        self.runtime.block_on(async {
            self.docker
                .start_container(&container.id, None::<StartContainerOptions>)
                .await
                .map_err(|e| {
                    ContainerError::CreationFailed(
                        config.name.clone().unwrap_or_default(),
                        e.to_string(),
                    )
                })
        })?;

        Ok(Box::new(PodmanContainer {
            docker: self.docker.clone(),
            container_id: container.id,
            runtime: self.runtime.handle().clone(),
        }))
    }

    fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
        self.runtime.block_on(async {
            self.docker
                .remove_container(
                    name,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await
                .map_err(|e| ContainerError::RemovalFailed(name.to_string(), e.to_string()))
        })
    }

    fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
        self.runtime.block_on(async {
            self.docker
                .stop_container(name, None)
                .await
                .map_err(|e| ContainerError::RemovalFailed(name.to_string(), e.to_string()))
        })
    }

    fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
        self.runtime.block_on(async {
            let info = self
                .docker
                .version()
                .await
                .map_err(|_| ContainerError::NotAvailable)?;

            Ok(HostInfo {
                os: info.os.unwrap_or_else(|| "linux".to_string()),
                arch: info.arch.unwrap_or_else(|| "amd64".to_string()),
                engine_version: info.version.unwrap_or_else(|| "unknown".to_string()),
            })
        })
    }
}
