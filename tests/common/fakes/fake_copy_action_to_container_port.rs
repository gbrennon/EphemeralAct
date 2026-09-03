#![allow(dead_code)]
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ephact::{
    application::{
        dtos::CopyActionToContainerRequest,
        ports::inbound::copy_action_to_container_port::CopyActionToContainerPort,
    },
    domain::errors::StepError,
};

/// Reports a prepared container-side directory, recording what it copied.
#[derive(Clone)]
pub struct FakeCopyActionToContainerPort {
    result: Result<String, String>,
    copied: Rc<RefCell<Vec<PathBuf>>>,
}

impl FakeCopyActionToContainerPort {
    pub fn returning(container_dir: &str) -> Self {
        Self {
            result: Ok(container_dir.to_string()),
            copied: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            copied: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn copied(&self) -> Vec<PathBuf> {
        self.copied.borrow().clone()
    }
}

impl CopyActionToContainerPort for FakeCopyActionToContainerPort {
    fn execute(&self, request: CopyActionToContainerRequest<'_>) -> Result<String, StepError> {
        self.copied
            .borrow_mut()
            .push(request.action_dir.to_path_buf());
        self.result.clone().map_err(StepError::new)
    }
}
