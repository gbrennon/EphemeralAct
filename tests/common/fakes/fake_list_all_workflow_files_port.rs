#![allow(dead_code)]
use ephact::infrastructure::workflows::list_all_workflow_files_port::ListAllWorkflowFilesPort;
use parking_lot::Mutex;
use std::path::PathBuf;

use ephact::application::dtos::{ListAllWorkflowFilesRequest, ListAllWorkflowFilesResponse};

/// Returns a prepared list of workflow files, or a prepared failure.
pub struct FakeListAllWorkflowFilesPort {
    result: Result<Vec<PathBuf>, String>,
    pub calls: Mutex<Vec<PathBuf>>,
}

impl FakeListAllWorkflowFilesPort {
    pub fn returning(files: Vec<PathBuf>) -> Self {
        Self {
            result: Ok(files),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl ListAllWorkflowFilesPort for FakeListAllWorkflowFilesPort {
    fn execute(
        &self,
        request: ListAllWorkflowFilesRequest<'_>,
    ) -> Result<ListAllWorkflowFilesResponse, Box<dyn std::error::Error>> {
        self.calls.lock().push(request.repo_path.to_path_buf());
        match &self.result {
            Ok(files) => Ok(ListAllWorkflowFilesResponse {
                workflow_files: files.clone(),
            }),
            Err(message) => Err(message.clone().into()),
        }
    }
}
