use crate::application::dtos::{ListAllWorkflowFilesRequest, ListAllWorkflowFilesResponse};

/// Inbound port for listing every workflow file of a repository.
pub trait ListAllWorkflowFilesPort {
    /// Lists the workflow files of every platform directory the repository has.
    fn execute(
        &self,
        request: ListAllWorkflowFilesRequest<'_>,
    ) -> Result<ListAllWorkflowFilesResponse, Box<dyn std::error::Error>>;
}
