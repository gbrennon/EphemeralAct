use crate::application::dtos::{ListWorkflowsRequest, ListWorkflowsResponse};

/// Inbound port for listing workflows in a repository.
pub trait ListWorkflowsPort {
    /// Discovers and lists all workflows found in the repository.
    fn execute(
        &self,
        request: ListWorkflowsRequest,
    ) -> Result<ListWorkflowsResponse, Box<dyn std::error::Error>>;
}
