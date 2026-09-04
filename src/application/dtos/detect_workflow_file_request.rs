use std::path::Path;

/// Request DTO for the
/// [`DetectWorkflowFilePort`](crate::application::ports::inbound::detect_workflow_file_port::DetectWorkflowFilePort)
/// inbound port.
pub struct DetectWorkflowFileRequest<'a> {
    /// Path to the repository whose workflow is detected.
    pub repo_path: &'a Path,
}
