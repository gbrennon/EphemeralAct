#![allow(dead_code)]
use ephact::application::ports::outbound::{
    ContainerConfig, ContainerError, ContainerPort, ContainerRuntimePort, HostInfo,
};

#[derive(Clone, Default)]
pub struct StubFailingContainerRuntime;

impl ContainerRuntimePort for StubFailingContainerRuntime {
    fn pull_image(&self, _image: &str, _platform: Option<&str>) -> Result<(), ContainerError> {
        Err(ContainerError::NotAvailable)
    }

    fn create_container(
        &self,
        _config: &ContainerConfig,
    ) -> Result<Box<dyn ContainerPort>, ContainerError> {
        Err(ContainerError::NotAvailable)
    }

    fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
        Err(ContainerError::RemovalFailed(
            name.to_string(),
            "removal failure".to_string(),
        ))
    }

    fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
        Err(ContainerError::Internal(format!("failed to stop {name}")))
    }

    fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
        Err(ContainerError::NotAvailable)
    }
}
