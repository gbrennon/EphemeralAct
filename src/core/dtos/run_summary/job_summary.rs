use serde::{Deserialize, Serialize};

/// Summary of a job within a workflow run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobSummary {
    pub job_id: String,
    pub name: Option<String>,
    pub steps: Vec<crate::core::dtos::run_summary::step_summary::StepSummary>,
    pub success: bool,
}
