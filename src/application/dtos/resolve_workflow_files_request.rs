use std::path::Path;

use crate::domain::ActRunConfig;

/// Request DTO for the
/// [`ResolveWorkflowFilesPort`](crate::application::ports::inbound::resolve_workflow_files_port::ResolveWorkflowFilesPort)
/// inbound port.
pub struct ResolveWorkflowFilesRequest<'a> {
    /// Configuration naming which workflows the run executes.
    pub config: &'a ActRunConfig,
    /// Path to the repository the workflows are resolved in.
    pub repo_path: &'a Path,
}
