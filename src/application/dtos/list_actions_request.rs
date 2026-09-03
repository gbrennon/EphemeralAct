use std::path::PathBuf;

/// Request DTO for the
/// [`ListActionsPort`](crate::application::ports::inbound::list_actions_port::ListActionsPort)
/// inbound port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListActionsRequest {
    /// Path to the repository whose workflows are searched for referenced
    /// actions.
    pub path: PathBuf,
}

impl ListActionsRequest {
    /// Creates a new list-actions request.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
