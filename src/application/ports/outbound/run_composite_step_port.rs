use crate::{
    application::{dtos::RunCompositeStepRequest, ports::outbound::ExecResult},
    domain::errors::StepError,
};

/// Inbound port for running one step of a composite action.
pub trait RunCompositeStepPort {
    /// Runs the step as a script, or as a nested action when it uses one.
    fn execute(&self, request: RunCompositeStepRequest<'_>) -> Result<ExecResult, StepError>;
}
