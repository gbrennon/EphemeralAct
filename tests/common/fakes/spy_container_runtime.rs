#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephact::application::ports::outbound::{
    ContainerConfig, ContainerError, ContainerPort, ContainerRuntimePort, HostInfo,
};

use super::stub_container::StubContainer;

#[derive(Clone, Default)]
pub struct SpyContainerRuntime {
    stopped_containers: Rc<RefCell<Vec<String>>>,
    removed_containers: Rc<RefCell<Vec<String>>>,
}

impl SpyContainerRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stopped_containers(&self) -> Vec<String> {
        self.stopped_containers.borrow().clone()
    }

    pub fn removed_containers(&self) -> Vec<String> {
        self.removed_containers.borrow().clone()
    }
}

impl ContainerRuntimePort for SpyContainerRuntime {
    fn pull_image(&self, _image: &str, _platform: Option<&str>) -> Result<(), ContainerError> {
        Ok(())
    }

    fn create_container(
        &self,
        _config: &ContainerConfig,
    ) -> Result<Box<dyn ContainerPort>, ContainerError> {
        Ok(Box::new(StubContainer))
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
