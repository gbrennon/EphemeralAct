use crate::application::dtos::ExecResult;
use crate::application::dtos::RunCompositeStepRequest;
use crate::domain::errors::StepError;

/// Inbound port for running one step of a composite action.
pub trait RunCompositeStepPort: Send + Sync {
    /// Runs the step as a script, or as a nested action when it uses one.
    fn execute(&self, request: RunCompositeStepRequest<'_>) -> Result<ExecResult, StepError>;
}
