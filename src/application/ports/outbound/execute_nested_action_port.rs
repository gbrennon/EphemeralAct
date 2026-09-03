use crate::{
    application::dtos::{ExecuteActionRequest, ExecuteActionResponse},
    domain::errors::StepError,
};

/// Outbound port for recursively executing nested actions inside a composite action.
pub trait ExecuteNestedActionPort {
    /// Runs a nested action and returns its response.
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError>;
}
