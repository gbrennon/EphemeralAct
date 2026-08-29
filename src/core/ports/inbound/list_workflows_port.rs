use crate::core::dtos::{ListWorkflowsRequest, ListWorkflowsResponse};

/// Inbound port for listing the workflows in a repository.
///
/// Implementing types discover the CI workflow files (e.g. under
/// `.forgejo/workflows/` or `.github/workflows/`), parse each one, and return
/// a raw summary of each in the response.
pub trait ListWorkflowsPort {
    /// Lists the workflows found in the repository and returns them in the
    /// response.
    fn execute(
        &self,
        request: ListWorkflowsRequest,
    ) -> Result<ListWorkflowsResponse, Box<dyn std::error::Error>>;
}
