use crate::application::dtos::{ResolveWorkflowFilesRequest, ResolveWorkflowFilesResponse};

/// Inbound port for resolving which workflow files a run executes.
pub trait ResolveWorkflowFilesPort {
    /// Resolves the run's workflow files from its configuration.
    fn execute(
        &self,
        request: ResolveWorkflowFilesRequest<'_>,
    ) -> Result<ResolveWorkflowFilesResponse, Box<dyn std::error::Error>>;
}
