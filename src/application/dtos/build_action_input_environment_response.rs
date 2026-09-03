use std::collections::HashMap;

/// Response DTO for the
/// [`BuildActionInputEnvironmentPort`](crate::application::ports::outbound::build_action_input_environment_port::BuildActionInputEnvironmentPort)
/// outbound port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildActionInputEnvironmentResponse {
    /// Environment variables with action inputs and path populated.
    pub env: HashMap<String, String>,
}
