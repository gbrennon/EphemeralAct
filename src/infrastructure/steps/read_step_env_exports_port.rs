use std::collections::HashMap;

use crate::application::dtos::ReadStepEnvExportsRequest;

/// Inbound port for reading the environment variables a step exported.
pub trait ReadStepEnvExportsPort: Send + Sync {
    /// Returns the variables the step exported, or none when it exported nothing.
    fn execute(&self, request: ReadStepEnvExportsRequest<'_>) -> HashMap<String, String>;
}
