use crate::{
    application::{dtos::RunNodeActionRequest, ports::outbound::ExecResult},
    domain::errors::StepError,
};

/// Inbound port for running a JavaScript action inside the job's container.
pub trait RunNodeActionPort {
    /// Copies the action in and runs its entry point with node.
    fn execute(&self, request: RunNodeActionRequest<'_>) -> Result<ExecResult, StepError>;
}
