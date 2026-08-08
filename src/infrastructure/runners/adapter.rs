use std::sync::Arc;

use super::{docker_runtime::DockerRuntime, podman_runtime::PodmanRuntime};
use crate::core::ports::outbound::{
    Container, ContainerConfig, ContainerError, ContainerRuntime, HostInfo,
};

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
            .or_else(|_| PodmanRuntime::new().map(ContainerRuntimeAdapter::Podman))
    }

    /// Human-readable name of the active runtime (e.g. "Docker", "Podman").
    fn runtime_name(&self) -> &str {
        match self {
            ContainerRuntimeAdapter::Docker(_) => "Docker",
            ContainerRuntimeAdapter::Podman(_) => "Podman",
        }
    }

    /// Replace "Docker" with the actual runtime name in error messages.
    /// Bollard always reports "Docker" even when talking to Podman.
    fn map_error(&self, err: ContainerError) -> ContainerError {
        let name = self.runtime_name();
        if name == "Docker" {
            return err;
        }
        let text = err.to_string().replace("Docker", name);
        ContainerError::Internal(text)
    }
}
impl ContainerRuntime for ContainerRuntimeAdapter {
    fn pull_image(&self, image: &str, platform: Option<&str>) -> Result<(), ContainerError> {
        let result = match self {
            ContainerRuntimeAdapter::Docker(rt) => rt.pull_image(image, platform),
            ContainerRuntimeAdapter::Podman(rt) => rt.pull_image(image, platform),
        };
        result.map_err(|e| self.map_error(e))
    }

    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn Container>, ContainerError> {
        let result = match self {
            ContainerRuntimeAdapter::Docker(rt) => rt.create_container(config),
            ContainerRuntimeAdapter::Podman(rt) => rt.create_container(config),
        };
        result.map_err(|e| self.map_error(e))
    }

    fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
        let result = match self {
            ContainerRuntimeAdapter::Docker(rt) => rt.remove_container(name),
            ContainerRuntimeAdapter::Podman(rt) => rt.remove_container(name),
        };
        result.map_err(|e| self.map_error(e))
    }

    fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
        let result = match self {
            ContainerRuntimeAdapter::Docker(rt) => rt.stop_container(name),
            ContainerRuntimeAdapter::Podman(rt) => rt.stop_container(name),
        };
        result.map_err(|e| self.map_error(e))
    }

    fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
        match self {
            ContainerRuntimeAdapter::Docker(rt) => rt.get_host_info(),
            ContainerRuntimeAdapter::Podman(rt) => rt.get_host_info(),
        }
    }
}

impl ContainerRuntime for Arc<ContainerRuntimeAdapter> {
    fn pull_image(&self, image: &str, platform: Option<&str>) -> Result<(), ContainerError> {
        self.as_ref().pull_image(image, platform)
    }
    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn Container>, ContainerError> {
        self.as_ref().create_container(config)
    }
    fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
        self.as_ref().remove_container(name)
    }

    fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
        self.as_ref().stop_container(name)
    }
    fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
        self.as_ref().get_host_info()
    }
}
