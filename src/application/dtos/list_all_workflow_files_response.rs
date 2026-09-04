use std::path::PathBuf;

/// Response DTO for the
/// [`ListAllWorkflowFilesPort`](crate::application::ports::inbound::list_all_workflow_files_port::ListAllWorkflowFilesPort)
/// inbound port.
#[derive(Debug)]
pub struct ListAllWorkflowFilesResponse {
    /// Every workflow file in the repository, `.forgejo` before `.github`.
    pub workflow_files: Vec<PathBuf>,
}
