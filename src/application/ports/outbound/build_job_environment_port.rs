use crate::application::dtos::{BuildJobEnvironmentRequest, BuildJobEnvironmentResponse};

/// Outbound port for building the environment variables a job runs with.
pub trait BuildJobEnvironmentPort {
    /// Returns the merged environment for the job's execution.
    fn execute(&self, request: BuildJobEnvironmentRequest<'_>) -> BuildJobEnvironmentResponse;
}
