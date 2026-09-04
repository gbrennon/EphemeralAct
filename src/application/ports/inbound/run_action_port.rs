use crate::{
    application::dtos::{ExecuteActionResponse, RunActionRequest},
    domain::errors::StepError,
};

/// Inbound port representing the entrypoint to run an action.
pub trait RunActionPort {
    /// Executes an action directly in the context of a container.
    fn execute(&self, request: RunActionRequest) -> Result<ExecuteActionResponse, StepError>;
}
