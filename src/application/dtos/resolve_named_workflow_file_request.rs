use std::path::Path;

/// Request DTO for the
/// [`ResolveNamedWorkflowFilePort`](crate::application::ports::inbound::resolve_named_workflow_file_port::ResolveNamedWorkflowFilePort)
/// inbound port.
pub struct ResolveNamedWorkflowFileRequest<'a> {
    /// Workflow the run was asked to execute, as named on the command line.
    pub workflow_name: &'a str,
    /// Path to the repository the workflow is looked up in.
    pub repo_path: &'a Path,
}
