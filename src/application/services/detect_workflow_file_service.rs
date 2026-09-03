use std::{error::Error, path::PathBuf};

use crate::application::{
    constants::WORKFLOW_DIRECTORIES,
    dtos::{DetectWorkflowFileRequest, ListWorkflowDirectoryRequest},
    ports::inbound::{
        detect_workflow_file_port::DetectWorkflowFilePort,
        list_workflow_directory_port::ListWorkflowDirectoryPort,
    },
};

/// Service that detects the workflow a repository runs when the caller names
/// none, preferring the Forgejo layout over the GitHub one.
pub struct DetectWorkflowFileService {
    directory_lister: Box<dyn ListWorkflowDirectoryPort>,
}

impl DetectWorkflowFileService {
    pub fn new(directory_lister: Box<dyn ListWorkflowDirectoryPort>) -> Self {
        Self { directory_lister }
    }
}

impl DetectWorkflowFilePort for DetectWorkflowFileService {
    fn execute(&self, request: DetectWorkflowFileRequest<'_>) -> Result<PathBuf, Box<dyn Error>> {
        for platform_dir in &WORKFLOW_DIRECTORIES {
            let workflows_dir = request.repo_path.join(platform_dir);
            if workflows_dir.exists() {
                return match self
                    .directory_lister
                    .execute(ListWorkflowDirectoryRequest {
                        directory: &workflows_dir,
                    })?
                    .workflow_files
                    .into_iter()
                    .next()
                {
                    Some(path) => Ok(path),
                    None => Err(format!("no workflow files found in {}/", platform_dir).into()),
                };
            }
        }

        Err("no workflows directory found (.forgejo/workflows/ or .github/workflows/)".into())
    }
}
