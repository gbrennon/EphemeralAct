#![allow(dead_code)]
use ephact::infrastructure::workflows::resolve_named_workflow_file_port::ResolveNamedWorkflowFilePort;
use parking_lot::Mutex;
use std::path::PathBuf;

use ephact::application::dtos::ResolveNamedWorkflowFileRequest;

/// Resolves every name to a prepared path, recording the names it was asked for.
pub struct FakeResolveNamedWorkflowFilePort {
    result: Result<PathBuf, String>,
    pub requested_names: Mutex<Vec<String>>,
}

impl FakeResolveNamedWorkflowFilePort {
    pub fn returning(path: PathBuf) -> Self {
        Self {
            result: Ok(path),
            requested_names: Mutex::new(Vec::new()),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            requested_names: Mutex::new(Vec::new()),
        }
    }
}

impl ResolveNamedWorkflowFilePort for FakeResolveNamedWorkflowFilePort {
    fn execute(
        &self,
        request: ResolveNamedWorkflowFileRequest<'_>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.requested_names
            .lock()
            .push(request.workflow_name.to_string());
        self.result.clone().map_err(Into::into)
    }
}
