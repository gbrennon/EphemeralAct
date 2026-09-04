use crate::infrastructure::workflows::{
    detect_workflow_file_port::DetectWorkflowFilePort,
    list_all_workflow_files_port::ListAllWorkflowFilesPort,
    resolve_named_workflow_file_port::ResolveNamedWorkflowFilePort,
    resolve_workflow_files_port::ResolveWorkflowFilesPort,
};
use std::error::Error;

use crate::application::dtos::{
    DetectWorkflowFileRequest, ListAllWorkflowFilesRequest, ResolveNamedWorkflowFileRequest,
    ResolveWorkflowFilesRequest, ResolveWorkflowFilesResponse,
};

/// Service that decides which workflow files a run executes: every workflow of
/// the repository, the one the caller named, or the detected default.
pub struct ResolveWorkflowFilesService {
    all_lister: Box<dyn ListAllWorkflowFilesPort>,
    named_resolver: Box<dyn ResolveNamedWorkflowFilePort>,
    detector: Box<dyn DetectWorkflowFilePort>,
}

impl ResolveWorkflowFilesService {
    pub fn new(
        all_lister: Box<dyn ListAllWorkflowFilesPort>,
        named_resolver: Box<dyn ResolveNamedWorkflowFilePort>,
        detector: Box<dyn DetectWorkflowFilePort>,
    ) -> Self {
        Self {
            all_lister,
            named_resolver,
            detector,
        }
    }
}

impl ResolveWorkflowFilesPort for ResolveWorkflowFilesService {
    fn execute(
        &self,
        request: ResolveWorkflowFilesRequest<'_>,
    ) -> Result<ResolveWorkflowFilesResponse, Box<dyn Error>> {
        let workflow_files = if request.config.all_workflows() {
            self.all_lister
                .execute(ListAllWorkflowFilesRequest {
                    repo_path: request.repo_path,
                })?
                .workflow_files
        } else if let Some(workflow) = request.config.workflow() {
            vec![
                self.named_resolver
                    .execute(ResolveNamedWorkflowFileRequest {
                        workflow_name: workflow.as_str(),
                        repo_path: request.repo_path,
                    })?,
            ]
        } else {
            vec![self.detector.execute(DetectWorkflowFileRequest {
                repo_path: request.repo_path,
            })?]
        };

        Ok(ResolveWorkflowFilesResponse { workflow_files })
    }
}
