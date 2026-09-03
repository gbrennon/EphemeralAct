use crate::application::dtos::{ExecuteJobRequest, JobExecution};

/// Inbound port for running one planned job.
pub trait ExecuteJobPort {
    /// Runs every step of the job inside a fresh container.
    fn execute(
        &self,
        request: ExecuteJobRequest<'_>,
    ) -> Result<JobExecution, Box<dyn std::error::Error>>;
}
