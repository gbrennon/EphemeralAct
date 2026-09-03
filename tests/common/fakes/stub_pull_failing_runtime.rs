#![allow(dead_code)]
use std::cell::RefCell;

use ephact::application::ports::outbound::{
    ContainerConfig, ContainerError, ContainerPort, ContainerRuntimePort, HostInfo,
};

use super::stub_container::StubContainer;

/// Runtime whose `pull_image` fails for the images it was told to reject,
/// recording every image it was asked to pull.
pub struct StubPullFailingRuntime {
    rejected_images: Vec<String>,
    pub pulled_images: RefCell<Vec<String>>,
}

impl StubPullFailingRuntime {
    pub fn rejecting(rejected_images: Vec<String>) -> Self {
        Self {
            rejected_images,
            pulled_images: RefCell::new(Vec::new()),
        }
    }
}

impl ContainerRuntimePort for StubPullFailingRuntime {
    fn pull_image(&self, image: &str, _platform: Option<&str>) -> Result<(), ContainerError> {
        self.pulled_images.borrow_mut().push(image.to_string());
        if self
            .rejected_images
            .iter()
            .any(|rejected| rejected == image)
        {
            return Err(ContainerError::NotAvailable);
        }
        Ok(())
    }

    fn create_container(
        &self,
        _config: &ContainerConfig,
    ) -> Result<Box<dyn ContainerPort>, ContainerError> {
        Ok(Box::new(StubContainer))
    }

    fn remove_container(&self, _name: &str) -> Result<(), ContainerError> {
        Ok(())
    }

    fn stop_container(&self, _name: &str) -> Result<(), ContainerError> {
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
