use crate::{
    application::dtos::{ExecuteActionRequest, ExecuteActionResponse},
    domain::errors::StepError,
};

/// Inbound port for asking the rest of the system to execute an action.
///
/// Distinct from
/// [`ExecuteActionPort`](crate::application::ports::inbound::execute_action_port::ExecuteActionPort):
/// this port requests an execution rather than performing one.
pub trait RequestActionExecutionPort {
    /// Requests the action's execution and returns the handler's outcome.
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError>;
}
