use std::collections::HashMap;

/// Request DTO for the
/// [`BuildActionInputEnvironmentPort`](crate::application::ports::outbound::build_action_input_environment_port::BuildActionInputEnvironmentPort)
/// inbound port.
pub struct BuildActionInputEnvironmentRequest<'a> {
    /// Environment the action runs with.
    pub env: &'a HashMap<String, String>,
    /// Inputs the action was called with.
    pub inputs: &'a HashMap<String, String>,
    /// Directory the action was copied to inside the container.
    pub action_path: &'a str,
}
