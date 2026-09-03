use std::error::Error;

use crate::application::{
    constants::WORKFLOW_DIRECTORIES,
    dtos::{
        ListAllWorkflowFilesRequest, ListAllWorkflowFilesResponse, ListWorkflowDirectoryRequest,
    },
    ports::inbound::{
        list_all_workflow_files_port::ListAllWorkflowFilesPort,
        list_workflow_directory_port::ListWorkflowDirectoryPort,
    },
};

/// Service that lists every workflow file of a repository, `.forgejo` first.
pub struct ListAllWorkflowFilesService {
    directory_lister: Box<dyn ListWorkflowDirectoryPort>,
}

impl ListAllWorkflowFilesService {
    pub fn new(directory_lister: Box<dyn ListWorkflowDirectoryPort>) -> Self {
        Self { directory_lister }
    }
}

impl ListAllWorkflowFilesPort for ListAllWorkflowFilesService {
    fn execute(
        &self,
        request: ListAllWorkflowFilesRequest<'_>,
    ) -> Result<ListAllWorkflowFilesResponse, Box<dyn Error>> {
        let mut workflows = Vec::new();
        for platform_dir in &WORKFLOW_DIRECTORIES {
            let workflows_dir = request.repo_path.join(platform_dir);
            if workflows_dir.exists() {
                workflows.extend(
                    self.directory_lister
                        .execute(ListWorkflowDirectoryRequest {
                            directory: &workflows_dir,
                        })?
                        .workflow_files,
                );
            }
        }
        if workflows.is_empty() {
            return Err(
                "no workflow files found in .forgejo/workflows/ or .github/workflows/".into(),
            );
        }
        Ok(ListAllWorkflowFilesResponse {
            workflow_files: workflows,
        })
    }
}
