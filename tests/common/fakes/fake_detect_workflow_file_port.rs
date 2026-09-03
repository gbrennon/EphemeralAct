#![allow(dead_code)]
use std::{cell::Cell, path::PathBuf};

use ephact::application::{
    dtos::DetectWorkflowFileRequest,
    ports::inbound::detect_workflow_file_port::DetectWorkflowFilePort,
};

/// Detects a prepared workflow file, recording whether it was consulted.
pub struct FakeDetectWorkflowFilePort {
    result: Result<PathBuf, String>,
    pub was_called: Cell<bool>,
}

impl FakeDetectWorkflowFilePort {
    pub fn returning(path: PathBuf) -> Self {
        Self {
            result: Ok(path),
            was_called: Cell::new(false),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            was_called: Cell::new(false),
        }
    }
}

impl DetectWorkflowFilePort for FakeDetectWorkflowFilePort {
    fn execute(
        &self,
        _request: DetectWorkflowFileRequest<'_>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.was_called.set(true);
        self.result.clone().map_err(Into::into)
    }
}
