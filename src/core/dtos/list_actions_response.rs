/// Response DTO for the
/// [`ListActionsPort`](crate::core::ports::inbound::list_actions_port::ListActionsPort)
/// inbound port.
///
/// Carries the action references (`uses:`) collected from the steps of every
/// workflow found in the repository.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListActionsResponse {
    /// The action references (`uses:`) used across the workflows, e.g.
    /// `actions/checkout@v4`, `./.forgejo/actions/my-action`, or
    /// `docker://node:20`.
    pub actions: Vec<String>,
}

impl ListActionsResponse {
    /// Creates a new list-actions response.
    pub fn new(actions: Vec<String>) -> Self {
        Self { actions }
    }
}
