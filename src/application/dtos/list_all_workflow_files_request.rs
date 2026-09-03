use std::path::Path;

/// Request DTO for the
/// [`ListAllWorkflowFilesPort`](crate::application::ports::outbound::list_all_workflow_files_port::ListAllWorkflowFilesPort)
/// inbound port.
pub struct ListAllWorkflowFilesRequest<'a> {
    /// Path to the repository whose workflow files are listed.
    pub repo_path: &'a Path,
}
