use serde::{Deserialize, Serialize};

/// Summary of a job within a workflow run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobSummary {
    pub job_id: String,
    pub name: Option<String>,
    pub matrix: Option<serde_json::Value>,
    pub steps: Vec<crate::core::dtos::run_summary::step_summary::StepSummary>,
    pub success: bool,
    pub completed_at: Option<String>,
}
