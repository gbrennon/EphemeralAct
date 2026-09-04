use crate::application::dtos::ExecResult;
use crate::application::dtos::RunShellStepRequest;
use crate::domain::errors::StepError;

/// Inbound port for running a step's shell script inside a container.
pub trait RunShellStepPort: Send + Sync {
    /// Runs the step's script and returns what the container reported.
    fn execute(&self, request: RunShellStepRequest<'_>) -> Result<ExecResult, StepError>;
}
