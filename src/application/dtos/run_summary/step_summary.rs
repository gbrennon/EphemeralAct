/// Summary of a step within a job run.
#[derive(Debug, Clone, PartialEq)]
pub struct StepSummary {
    pub name: String,
    pub step_type: crate::application::dtos::run_summary::step_type::StepType,
    pub exit_code: Option<i64>,
    pub continue_on_error: bool,
    pub duration: std::time::Duration,
    pub stdout: String,
    pub stderr: String,
}
