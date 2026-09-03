use std::collections::HashMap;

use crate::application::dtos::BuildJobEnvironmentRequest;

/// Inbound port for building the environment a job's container runs with.
pub trait BuildJobEnvironmentPort {
    /// Merges the workflow and job environments and adds the runner's own
    /// variables.
    fn execute(&self, request: BuildJobEnvironmentRequest<'_>) -> HashMap<String, String>;
}
