#![allow(dead_code)]
use std::{cell::RefCell, path::PathBuf};

use ephact::application::{
    dtos::{ListAllWorkflowFilesRequest, ListAllWorkflowFilesResponse},
    ports::inbound::list_all_workflow_files_port::ListAllWorkflowFilesPort,
};

/// Returns a prepared list of workflow files, or a prepared failure.
pub struct FakeListAllWorkflowFilesPort {
    result: Result<Vec<PathBuf>, String>,
    pub calls: RefCell<Vec<PathBuf>>,
}

impl FakeListAllWorkflowFilesPort {
    pub fn returning(files: Vec<PathBuf>) -> Self {
        Self {
            result: Ok(files),
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl ListAllWorkflowFilesPort for FakeListAllWorkflowFilesPort {
    fn execute(
        &self,
        request: ListAllWorkflowFilesRequest<'_>,
    ) -> Result<ListAllWorkflowFilesResponse, Box<dyn std::error::Error>> {
        self.calls
            .borrow_mut()
            .push(request.repo_path.to_path_buf());
        match &self.result {
            Ok(files) => Ok(ListAllWorkflowFilesResponse {
                workflow_files: files.clone(),
            }),
            Err(message) => Err(message.clone().into()),
        }
    }
}
