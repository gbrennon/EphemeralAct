use std::cell::RefCell;

use ephemeral_act::core::ports::outbound::{
    Container, ContainerConfig, ContainerError, ContainerRuntime, ExecResult, HostInfo,
};

use super::fake_container::FakeContainer;

/// Fake container runtime that records calls and returns pre-configured
/// results.
pub(super) struct FakeRuntime {
    pub(super) pulled_images: RefCell<Vec<String>>,
    pub(super) created_containers: RefCell<Vec<ContainerConfig>>,
    pub(super) exec_results: RefCell<Vec<ExecResult>>,
    pub(super) removed_containers: RefCell<Vec<String>>,
    pub(super) stopped_containers: RefCell<Vec<String>>,
}

impl FakeRuntime {
    pub(super) fn new() -> Self {
        Self {
            pulled_images: RefCell::new(vec![]),
            created_containers: RefCell::new(vec![]),
            exec_results: RefCell::new(vec![]),
            removed_containers: RefCell::new(vec![]),
            stopped_containers: RefCell::new(vec![]),
        }
    }
}

impl ContainerRuntime for FakeRuntime {
    fn pull_image(&self, image: &str, _platform: Option<&str>) -> Result<(), ContainerError> {
        self.pulled_images.borrow_mut().push(image.to_string());
        Ok(())
    }

    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn Container>, ContainerError> {
        self.created_containers.borrow_mut().push(config.clone());
        Ok(Box::new(FakeContainer {
            exec_results: self.exec_results.clone(),
        }))
    }

    fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
        self.removed_containers.borrow_mut().push(name.to_string());
        Ok(())
    }

    fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
        self.stopped_containers.borrow_mut().push(name.to_string());
        Ok(())
    }

    fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
        Ok(HostInfo {
            os: "linux".into(),
            arch: "amd64".into(),
            engine_version: "1.0".into(),
        })
    }
}
