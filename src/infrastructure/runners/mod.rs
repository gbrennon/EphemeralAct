pub mod docker;
pub mod podman;

use crate::core::ports::outbound::{Container, ContainerConfig, ContainerError, ContainerRuntime, HostInfo};

use self::docker::DockerRuntime;
use self::podman::PodmanRuntime;

/// Map a GitHub Actions platform label to the corresponding container image.
///
/// Uses the `catthehacker/ubuntu` images which are designed for local act-based
/// workflow execution. Unknown platforms are returned as-is (assumed to be a
/// user-provided image name).
pub fn map_platform_to_image(platform: &str) -> &str {
    match platform {
        "ubuntu-latest" => "catthehacker/ubuntu:act-latest",
        "ubuntu-24.04" => "catthehacker/ubuntu:act-24.04",
        "ubuntu-22.04" => "catthehacker/ubuntu:act-22.04",
        "ubuntu-20.04" => "catthehacker/ubuntu:act-20.04",
        other => other,
    }
}

/// Enum dispatch over available container runtimes.
///
/// Probes Docker first, then Podman. Implements [`ContainerRuntime`] by
/// delegating to the inner adapter.
pub enum ContainerRuntimeAdapter {
    Docker(DockerRuntime),
    Podman(PodmanRuntime),
}

impl ContainerRuntimeAdapter {
    /// Auto-detect the available container runtime.
    ///
    /// Tries Docker first, then falls back to Podman. Returns
    /// `ContainerError::NotAvailable` if neither is reachable.
    pub fn detect() -> Result<Self, ContainerError> {
        DockerRuntime::new()
            .map(ContainerRuntimeAdapter::Docker)
            .or_else(|_| {
                PodmanRuntime::new().map(ContainerRuntimeAdapter::Podman)
            })
    }
}

impl ContainerRuntime for ContainerRuntimeAdapter {
    fn pull_image(&self, image: &str, platform: Option<&str>) -> Result<(), ContainerError> {
        match self {
            ContainerRuntimeAdapter::Docker(rt) => rt.pull_image(image, platform),
            ContainerRuntimeAdapter::Podman(rt) => rt.pull_image(image, platform),
        }
    }

    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn Container>, ContainerError> {
        match self {
            ContainerRuntimeAdapter::Docker(rt) => rt.create_container(config),
            ContainerRuntimeAdapter::Podman(rt) => rt.create_container(config),
        }
    }

    fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
        match self {
            ContainerRuntimeAdapter::Docker(rt) => rt.get_host_info(),
            ContainerRuntimeAdapter::Podman(rt) => rt.get_host_info(),
        }
    }
}