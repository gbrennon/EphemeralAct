use crate::{
    application::dtos::{ExecuteActionResponse, RunCompositeActionRequest},
    domain::errors::StepError,
};

/// Inbound port for running a composite action's steps.
pub trait RunCompositeActionPort {
    /// Runs every step of the composite action in order.
    fn execute(
        &self,
        request: RunCompositeActionRequest<'_>,
    ) -> Result<ExecuteActionResponse, StepError>;
}
