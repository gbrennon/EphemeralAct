use std::path::PathBuf;

/// Response DTO for the
/// [`ResolveWorkflowFilesPort`](crate::application::ports::inbound::resolve_workflow_files_port::ResolveWorkflowFilesPort)
/// inbound port.
#[derive(Debug)]
pub struct ResolveWorkflowFilesResponse {
    /// Workflow files the run executes, in execution order.
    pub workflow_files: Vec<PathBuf>,
}
