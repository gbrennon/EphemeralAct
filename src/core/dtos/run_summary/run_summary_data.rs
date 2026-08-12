/// Workflow run summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RunSummary {
    pub name: Option<String>,
    pub job_summaries: Vec<crate::core::dtos::run_summary::job_summary::JobSummary>,
    pub success: bool,
    pub total_duration: std::time::Duration,
}
