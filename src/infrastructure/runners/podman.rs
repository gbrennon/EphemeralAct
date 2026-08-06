use std::collections::HashMap;

use bollard::container::LogOutput;
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, DownloadFromContainerOptionsBuilder,
    InspectContainerOptions, RemoveContainerOptions, StartContainerOptions,
    UploadToContainerOptionsBuilder,
};
use bollard::Docker;
use bytes::Bytes;
use futures_util::StreamExt;
use tokio::runtime::Runtime;

use crate::core::ports::outbound::{
    Container, ContainerConfig, ContainerError, ContainerRuntime, ExecResult, FileEntry, HostInfo,
    RunnerContext,
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
            .or_else(|_| {
                Docker::connect_with_unix(root_socket, 120, bollard::API_DEFAULT_VERSION)
            })
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

/// A running Podman container, created by [`PodmanRuntime`].
struct PodmanContainer {
    docker: Docker,
    container_id: String,
    runtime: tokio::runtime::Handle,
}

impl Container for PodmanContainer {
    fn exec(&self, cmd: &[String], workdir: Option<&str>) -> Result<ExecResult, ContainerError> {
        let exec_config = CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            cmd: Some(cmd.to_vec()),
            working_dir: workdir.map(|s| s.to_string()),
            ..Default::default()
        };

        self.runtime.block_on(async {
            let exec = self
                .docker
                .create_exec(&self.container_id, exec_config)
                .await
                .map_err(|e| {
                    ContainerError::ExecutionFailed(self.container_id.clone(), e.to_string())
                })?;

            let output = self
                .docker
                .start_exec(&exec.id, None::<StartExecOptions>)
                .await
                .map_err(|e| {
                    ContainerError::ExecutionFailed(self.container_id.clone(), e.to_string())
                })?;

            match output {
                StartExecResults::Attached { mut output, .. } => {
                    let mut stdout = String::new();
                    let mut stderr = String::new();
                    while let Some(chunk) = output.next().await {
                        match chunk {
                            Ok(LogOutput::StdOut { message }) => {
                                stdout.push_str(&String::from_utf8_lossy(&message));
                            }
                            Ok(LogOutput::StdErr { message }) => {
                                stderr.push_str(&String::from_utf8_lossy(&message));
                            }
                            Ok(_) => {}
                            Err(e) => {
                                return Err(ContainerError::ExecutionFailed(
                                    self.container_id.clone(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                    Ok(ExecResult {
                        exit_code: 0,
                        stdout,
                        stderr,
                    })
                }
                StartExecResults::Detached => Ok(ExecResult {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                }),
            }
        })
    }

    fn copy_to(
        &self,
        container_path: &str,
        entries: &[FileEntry],
    ) -> Result<(), ContainerError> {
        let mut tar_builder = tar::Builder::new(Vec::new());

        for entry in entries {
            let mut header = tar::Header::new_gnu();
            header
                .set_path(&entry.path)
                .map_err(|e| ContainerError::CopyFailed(self.container_id.clone(), e.to_string()))?;
            header.set_size(entry.content.len() as u64);
            header.set_mode(entry.mode);
            header.set_cksum();

            tar_builder
                .append(&header, entry.content.as_slice())
                .map_err(|e| ContainerError::CopyFailed(self.container_id.clone(), e.to_string()))?;
        }

        let tar_data = tar_builder
            .into_inner()
            .map_err(|e| ContainerError::CopyFailed(self.container_id.clone(), e.to_string()))?;

        let upload_options = UploadToContainerOptionsBuilder::new()
            .path(container_path)
            .build();

        self.runtime.block_on(async {
            self.docker
                .upload_to_container(
                    &self.container_id,
                    Some(upload_options),
                    bollard::body_full(Bytes::from(tar_data)),
                )
                .await
                .map_err(|e| ContainerError::CopyFailed(self.container_id.clone(), e.to_string()))
        })?;

        Ok(())
    }

    fn copy_from(&self, container_path: &str) -> Result<Vec<FileEntry>, ContainerError> {
        let download_options = DownloadFromContainerOptionsBuilder::new()
            .path(container_path)
            .build();

        self.runtime.block_on(async {
            let mut stream = self.docker.download_from_container(
                &self.container_id,
                Some(download_options),
            );

            let mut tar_bytes = Vec::new();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => tar_bytes.extend_from_slice(&chunk),
                    Err(e) => {
                        return Err(ContainerError::CopyFailed(
                            self.container_id.clone(),
                            e.to_string(),
                        ));
                    }
                }
            }

            let mut archive = tar::Archive::new(tar_bytes.as_slice());
            let mut entries = Vec::new();

            for entry_result in archive
                .entries()
                .map_err(|e| ContainerError::CopyFailed(self.container_id.clone(), e.to_string()))?
            {
                let mut entry = entry_result.map_err(|e| {
                    ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                })?;

                let path = entry
                    .path()
                    .map_err(|e| {
                        ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                    })?
                    .to_string_lossy()
                    .to_string();

                let mode = entry.header().mode().map_err(|e| {
                    ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                })?;

                let mut content = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut content).map_err(|e| {
                    ContainerError::CopyFailed(self.container_id.clone(), e.to_string())
                })?;

                entries.push(FileEntry {
                    path,
                    content,
                    mode,
                });
            }

            Ok(entries)
        })
    }

    fn remove(&self) -> Result<(), ContainerError> {
        self.runtime.block_on(async {
            self.docker
                .remove_container(
                    &self.container_id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await
                .map_err(|e| {
                    ContainerError::RemovalFailed(self.container_id.clone(), e.to_string())
                })
        })
    }

    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
        self.runtime.block_on(async {
            let info = self
                .docker
                .inspect_container(&self.container_id, None::<InspectContainerOptions>)
                .await
                .map_err(|_| ContainerError::NotFound(self.container_id.clone()))?;

            let env_map: HashMap<String, String> = info
                .config
                .and_then(|c| c.env)
                .unwrap_or_default()
                .iter()
                .filter_map(|kv| {
                    let mut parts = kv.splitn(2, '=');
                    Some((
                        parts.next()?.to_string(),
                        parts.next().unwrap_or("").to_string(),
                    ))
                })
                .collect();

            Ok(RunnerContext {
                workspace: "/github/workspace".to_string(),
                home: "/github/home".to_string(),
                action_path: "/github/action".to_string(),
                temp: "/github/temp".to_string(),
                tool_cache: "/github/tool_cache".to_string(),
                env: env_map,
            })
        })
    }
}