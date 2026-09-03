use crate::application::dtos::{
    BuildActionInputEnvironmentRequest, BuildActionInputEnvironmentResponse,
};

/// Outbound port for exposing an action's inputs as environment variables.
pub trait BuildActionInputEnvironmentPort {
    /// Returns the environment with action inputs and action path configured.
    fn execute(
        &self,
        request: BuildActionInputEnvironmentRequest<'_>,
    ) -> BuildActionInputEnvironmentResponse;
}
