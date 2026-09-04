use crate::application::dtos::{ListActionsRequest, ListActionsResponse};

/// Inbound port for listing actions referenced across workflows.
pub trait ListActionsPort {
    /// Discovers and lists all actions used in the repository's workflows.
    fn execute(
        &self,
        request: ListActionsRequest,
    ) -> Result<ListActionsResponse, Box<dyn std::error::Error>>;
}
