use crate::application::dtos::JobSummary;

#[derive(Debug, Clone)]
pub struct WorkflowExecution {
    /// Name reported for the execution as a whole.
    pub workflow_name: String,
    /// Summary of every job that ran, in execution order.
    pub job_summaries: Vec<JobSummary>,
    /// Name of every container created during the execution.
    pub container_names: Vec<String>,
    /// Whether every job of the execution succeeded.
    pub success: bool,
}
