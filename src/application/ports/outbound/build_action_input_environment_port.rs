use std::collections::HashMap;

use crate::application::dtos::BuildActionInputEnvironmentRequest;

/// Inbound port for exposing an action's inputs as environment variables.
pub trait BuildActionInputEnvironmentPort {
    /// Returns the environment with `GITHUB_ACTION_PATH` and the `INPUT_*`
    /// variables a runner exposes.
    fn execute(&self, request: BuildActionInputEnvironmentRequest<'_>) -> HashMap<String, String>;
}
