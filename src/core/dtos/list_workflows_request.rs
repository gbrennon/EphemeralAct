use std::path::PathBuf;

/// Request DTO for the
/// [`ListWorkflowsPort`](crate::core::ports::inbound::list_workflows_port::ListWorkflowsPort)
/// inbound port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListWorkflowsRequest {
    /// Path to the repository whose workflows are to be listed.
    pub path: PathBuf,
}

impl ListWorkflowsRequest {
    /// Creates a new list-workflows request.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
