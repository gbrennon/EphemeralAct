/// Workflow run summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RunSummary {
    pub name: String,
    pub job_summaries: Vec<crate::core::dtos::run_summary::job_summary::JobSummary>,
    pub success: bool,
    pub duration: std::time::Duration,
}
