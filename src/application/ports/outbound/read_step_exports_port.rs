use crate::application::dtos::{ReadStepExportsRequest, StepExports};

/// Inbound port for reading everything a step exported to later steps.
pub trait ReadStepExportsPort: Send + Sync {
    /// Reads the step's `PATH` and environment exports.
    fn execute(&self, request: ReadStepExportsRequest<'_>) -> StepExports;
}
