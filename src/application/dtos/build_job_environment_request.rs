use std::collections::HashMap;

use crate::domain::workflow::Workflow;

/// Request DTO for the
/// [`BuildJobEnvironmentPort`](crate::application::ports::outbound::build_job_environment_port::BuildJobEnvironmentPort)
/// outbound port.
pub struct BuildJobEnvironmentRequest<'a> {
    /// Workflow the job belongs to, for its workflow-level environment.
    pub workflow: &'a Workflow,
    /// Environment the job itself declared.
    pub job_env: &'a HashMap<String, String>,
}
