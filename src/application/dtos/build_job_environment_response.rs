use std::collections::HashMap;

/// Response DTO for the
/// [`BuildJobEnvironmentPort`](crate::application::ports::outbound::build_job_environment_port::BuildJobEnvironmentPort)
/// outbound port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildJobEnvironmentResponse {
    /// Environment variables for the job container.
    pub env: HashMap<String, String>,
}
