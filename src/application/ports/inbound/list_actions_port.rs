use crate::application::dtos::{ListActionsRequest, ListActionsResponse};

/// Inbound port for listing the actions referenced in a repository's workflows.
///
/// Implementing types scan the repository's workflow files and collect the
/// action references (`uses:`) used by their steps.
pub trait ListActionsPort {
    /// Lists the action references used across the repository's workflows and
    /// returns them in the response.
    fn execute(
        &self,
        request: ListActionsRequest,
    ) -> Result<ListActionsResponse, Box<dyn std::error::Error>>;
}
