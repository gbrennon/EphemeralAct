use crate::infrastructure::workflows::resolve_named_workflow_file_port::ResolveNamedWorkflowFilePort;
use std::{error::Error, path::PathBuf};

use crate::{
    application::dtos::ResolveNamedWorkflowFileRequest,
    infrastructure::workflows::workflow_directories::WORKFLOW_DIRECTORIES,
};

/// Service that resolves the file of a workflow the caller named.
///
/// A name is first taken as a path relative to the repository root, then looked
/// up inside each supported platform directory in turn.
pub struct ResolveNamedWorkflowFileService;

impl ResolveNamedWorkflowFileService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResolveNamedWorkflowFileService {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolveNamedWorkflowFilePort for ResolveNamedWorkflowFileService {
    fn execute(
        &self,
        request: ResolveNamedWorkflowFileRequest<'_>,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let direct = request.repo_path.join(request.workflow_name);
        if direct.exists() {
            return Ok(direct);
        }
        for platform_dir in &WORKFLOW_DIRECTORIES {
            let path = request
                .repo_path
                .join(platform_dir)
                .join(request.workflow_name);
            if path.exists() {
                return Ok(path);
            }
        }
        Err(format!("workflow file not found: {}", request.workflow_name).into())
    }
}
