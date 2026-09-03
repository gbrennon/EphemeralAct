use std::path::Path;

/// Request DTO for the
/// [`LoadWorkflowPort`](crate::application::ports::outbound::load_workflow_port::LoadWorkflowPort)
/// inbound port.
pub struct LoadWorkflowRequest<'a> {
    /// Workflow file to read and parse.
    pub workflow_file: &'a Path,
}
