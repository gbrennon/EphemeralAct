#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc, sync::Arc};

use ephact::application::{
    dtos::CreateJobContainerRequest,
    ports::{inbound::create_job_container_port::CreateJobContainerPort, outbound::ContainerPort},
};

use super::stub_container::StubContainer;

/// Records the creation requests it receives and hands back a stub container.
///
/// Shares its recordings across clones, so a test can keep inspecting it after
/// injecting it into a service.
#[derive(Clone, Default)]
pub struct FakeCreateJobContainerPort {
    images: Rc<RefCell<Vec<String>>>,
    container_names: Rc<RefCell<Vec<String>>>,
    legacy_container_names: Rc<RefCell<Vec<String>>>,
}

impl FakeCreateJobContainerPort {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn images(&self) -> Vec<String> {
        self.images.borrow().clone()
    }

    pub fn container_names(&self) -> Vec<String> {
        self.container_names.borrow().clone()
    }

    pub fn legacy_container_names(&self) -> Vec<String> {
        self.legacy_container_names.borrow().clone()
    }
}

impl CreateJobContainerPort for FakeCreateJobContainerPort {
    fn execute(
        &self,
        request: CreateJobContainerRequest<'_>,
    ) -> Result<Arc<dyn ContainerPort>, Box<dyn std::error::Error>> {
        self.images.borrow_mut().push(request.image.to_string());
        self.container_names
            .borrow_mut()
            .push(request.container_name.to_string());
        self.legacy_container_names
            .borrow_mut()
            .push(request.legacy_container_name.to_string());
        Ok(Arc::new(StubContainer))
    }
}
