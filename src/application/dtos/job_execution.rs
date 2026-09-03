use crate::application::dtos::JobSummary;

/// Outcome of running one job, with the container it ran in.
pub struct JobExecution {
    /// Summary reported for the job.
    pub job_summary: JobSummary,
    /// Name of the container the job ran in.
    pub container_name: String,
}
