use crate::application::dtos::{SummarizeStepRequest, SummarizedStep};

/// Inbound port for turning a step's outcome into its run-summary entry.
pub trait SummarizeStepPort {
    /// Summarises the step and reports whether it fails the job.
    fn execute(&self, request: SummarizeStepRequest<'_>) -> SummarizedStep;
}
