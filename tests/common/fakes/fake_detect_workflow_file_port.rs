#![allow(dead_code)]
use ephact::infrastructure::workflows::detect_workflow_file_port::DetectWorkflowFilePort;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use ephact::application::dtos::DetectWorkflowFileRequest;

/// Detects a prepared workflow file, recording whether it was consulted.
pub struct FakeDetectWorkflowFilePort {
    result: Result<PathBuf, String>,
    pub was_called: AtomicBool,
}

impl FakeDetectWorkflowFilePort {
    pub fn returning(path: PathBuf) -> Self {
        Self {
            result: Ok(path),
            was_called: AtomicBool::new(false),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            was_called: AtomicBool::new(false),
        }
    }
}

impl DetectWorkflowFilePort for FakeDetectWorkflowFilePort {
    fn execute(
        &self,
        _request: DetectWorkflowFileRequest<'_>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.was_called.store(true, Ordering::SeqCst);
        self.result.clone().map_err(Into::into)
    }
}
