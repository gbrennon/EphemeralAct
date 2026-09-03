#![allow(dead_code)]
use std::{cell::RefCell, path::PathBuf};

use ephact::application::{
    dtos::ResolveNamedWorkflowFileRequest,
    ports::outbound::resolve_named_workflow_file_port::ResolveNamedWorkflowFilePort,
};

/// Resolves every name to a prepared path, recording the names it was asked for.
pub struct FakeResolveNamedWorkflowFilePort {
    result: Result<PathBuf, String>,
    pub requested_names: RefCell<Vec<String>>,
}

impl FakeResolveNamedWorkflowFilePort {
    pub fn returning(path: PathBuf) -> Self {
        Self {
            result: Ok(path),
            requested_names: RefCell::new(Vec::new()),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            requested_names: RefCell::new(Vec::new()),
        }
    }
}

impl ResolveNamedWorkflowFilePort for FakeResolveNamedWorkflowFilePort {
    fn execute(
        &self,
        request: ResolveNamedWorkflowFileRequest<'_>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.requested_names
            .borrow_mut()
            .push(request.workflow_name.to_string());
        self.result.clone().map_err(Into::into)
    }
}
