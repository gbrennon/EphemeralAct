use crate::{application::dtos::BuildStepContextRequest, domain::expression::EvalContext};

/// Inbound port for building the context one step's expressions resolve against.
pub trait BuildStepContextPort: Send + Sync {
    /// Returns the run context with its `env` mirroring the step's environment.
    fn execute(&self, request: BuildStepContextRequest<'_>) -> EvalContext;
}
