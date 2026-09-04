use crate::application::dtos::{ListWorkflowDirectoryRequest, ListWorkflowDirectoryResponse};

/// Inbound port for listing the workflow files held directly by one directory.
pub trait ListWorkflowDirectoryPort: Send + Sync {
    /// Lists the `.yml`/`.yaml` files of the requested directory, sorted by path.
    fn execute(
        &self,
        request: ListWorkflowDirectoryRequest<'_>,
    ) -> Result<ListWorkflowDirectoryResponse, Box<dyn std::error::Error>>;
}
