#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc, sync::Arc};

use ephact::application::{
    dtos::{PrepareJobContainerRequest, PreparedJobContainer},
    ports::inbound::prepare_job_container_port::PrepareJobContainerPort,
};

use super::stub_container::StubContainer;

/// Prepares a stub container under a prepared name, or fails as configured.
#[derive(Clone)]
pub struct FakePrepareJobContainerPort {
    container_name: String,
    failure: Option<String>,
    job_ids: Rc<RefCell<Vec<String>>>,
}

impl FakePrepareJobContainerPort {
    pub fn named(container_name: &str) -> Self {
        Self {
            container_name: container_name.to_string(),
            failure: None,
            job_ids: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            container_name: String::new(),
            failure: Some(message.to_string()),
            job_ids: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn job_ids(&self) -> Vec<String> {
        self.job_ids.borrow().clone()
    }
}

impl PrepareJobContainerPort for FakePrepareJobContainerPort {
    fn execute(
        &self,
        request: PrepareJobContainerRequest<'_>,
    ) -> Result<PreparedJobContainer, Box<dyn std::error::Error>> {
        self.job_ids.borrow_mut().push(request.job_id.to_string());
        if let Some(message) = &self.failure {
            return Err(message.clone().into());
        }
        Ok(PreparedJobContainer {
            container: Arc::new(StubContainer),
            container_name: self.container_name.clone(),
        })
    }
}
