use crate::application::dtos::{PrepareJobContainerRequest, PreparedJobContainer};

/// Inbound port for preparing the container a job's steps run in.
pub trait PrepareJobContainerPort: Send + Sync {
    /// Pulls the job's image and creates the container to run its steps in.
    fn execute(
        &self,
        request: PrepareJobContainerRequest<'_>,
    ) -> Result<PreparedJobContainer, Box<dyn std::error::Error>>;
}
