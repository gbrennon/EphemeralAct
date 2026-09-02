#![allow(dead_code)]
use std::{ops::Deref, rc::Rc};

use ephemeral_act::core::ports::outbound::{
    ContainerConfig, ContainerError, ContainerPort, ContainerRuntimePort, HostInfo,
};

use super::fake_runtime::FakeRuntime;

/// A [`FakeRuntime`] a test can keep inspecting after injecting it into a
/// service, the way the production adapter is shared through an `Arc`.
#[derive(Clone)]
pub struct SharedFakeRuntime(Rc<FakeRuntime>);

impl SharedFakeRuntime {
    pub fn new() -> Self {
        Self(Rc::new(FakeRuntime::new()))
    }
}

impl Deref for SharedFakeRuntime {
    type Target = FakeRuntime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ContainerRuntimePort for SharedFakeRuntime {
    fn pull_image(&self, image: &str, platform: Option<&str>) -> Result<(), ContainerError> {
        self.0.pull_image(image, platform)
    }

    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn ContainerPort>, ContainerError> {
        self.0.create_container(config)
    }

    fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
        self.0.remove_container(name)
    }

    fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
        self.0.stop_container(name)
    }

    fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
        self.0.get_host_info()
    }
}
