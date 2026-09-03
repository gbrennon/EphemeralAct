use crate::application::dtos::StepSummary;

/// Summary of one executed step, and whether it fails the job it belongs to.
pub struct SummarizedStep {
    /// Summary reported for the step.
    pub summary: StepSummary,
    /// Whether this step's outcome fails the job.
    pub fails_job: bool,
}
