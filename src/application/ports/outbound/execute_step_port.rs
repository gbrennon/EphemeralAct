use crate::{
    application::dtos::{ExecuteStepRequest, ExecutedStep},
    domain::errors::StepError,
};

/// Inbound port for executing one step of a job.
pub trait ExecuteStepPort {
    /// Resolves the step's expressions and runs it as a script or an action.
    fn execute(&self, request: ExecuteStepRequest<'_>) -> Result<ExecutedStep, StepError>;
}
