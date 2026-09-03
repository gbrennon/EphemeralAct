use std::path::PathBuf;

/// Response DTO for the
/// [`ListWorkflowDirectoryPort`](crate::application::ports::inbound::list_workflow_directory_port::ListWorkflowDirectoryPort)
/// inbound port.
#[derive(Debug)]
pub struct ListWorkflowDirectoryResponse {
    /// Workflow files found directly inside the directory, sorted by path.
    pub workflow_files: Vec<PathBuf>,
}
