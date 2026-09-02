use crate::core::dtos::JobSummary;

/// Aggregated outcome of executing the jobs of one or more workflow files.
pub(crate) struct WorkflowExecution {
    pub(crate) workflow_name: String,
    pub(crate) job_summaries: Vec<JobSummary>,
    pub(crate) container_names: Vec<String>,
    pub(crate) success: bool,
}
