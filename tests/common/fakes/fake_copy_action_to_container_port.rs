#![allow(dead_code)]
use ephact::infrastructure::actions::copy_action_to_container_port::CopyActionToContainerPort;
use parking_lot::Mutex;
use std::{path::PathBuf, sync::Arc};

use ephact::{application::dtos::CopyActionToContainerRequest, domain::errors::StepError};

/// Reports a prepared container-side directory, recording what it copied.
#[derive(Clone)]
pub struct FakeCopyActionToContainerPort {
    result: Result<String, String>,
    copied: Arc<Mutex<Vec<PathBuf>>>,
}

impl FakeCopyActionToContainerPort {
    pub fn returning(container_dir: &str) -> Self {
        Self {
            result: Ok(container_dir.to_string()),
            copied: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            copied: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn copied(&self) -> Vec<PathBuf> {
        self.copied.lock().clone()
    }
}

impl CopyActionToContainerPort for FakeCopyActionToContainerPort {
    fn execute(&self, request: CopyActionToContainerRequest<'_>) -> Result<String, StepError> {
        self.copied.lock().push(request.action_dir.to_path_buf());
        self.result.clone().map_err(StepError::new)
    }
}
