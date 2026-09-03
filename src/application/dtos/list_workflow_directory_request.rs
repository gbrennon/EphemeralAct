use std::path::Path;

/// Request DTO for the
/// [`ListWorkflowDirectoryPort`](crate::application::ports::inbound::list_workflow_directory_port::ListWorkflowDirectoryPort)
/// inbound port.
pub struct ListWorkflowDirectoryRequest<'a> {
    /// Directory whose workflow files are listed.
    pub directory: &'a Path,
}
