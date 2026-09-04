use crate::infrastructure::workflows::list_workflow_directory_port::ListWorkflowDirectoryPort;
use std::{error::Error, fs::read_dir};

use crate::application::dtos::{ListWorkflowDirectoryRequest, ListWorkflowDirectoryResponse};

/// Service that lists the workflow files held directly by one directory.
pub struct ListWorkflowDirectoryService;

impl ListWorkflowDirectoryService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListWorkflowDirectoryService {
    fn default() -> Self {
        Self::new()
    }
}

impl ListWorkflowDirectoryPort for ListWorkflowDirectoryService {
    fn execute(
        &self,
        request: ListWorkflowDirectoryRequest<'_>,
    ) -> Result<ListWorkflowDirectoryResponse, Box<dyn Error>> {
        let mut files = Vec::new();
        for entry in read_dir(request.directory)? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
            {
                files.push(path);
            }
        }
        files.sort();
        Ok(ListWorkflowDirectoryResponse {
            workflow_files: files,
        })
    }
}
