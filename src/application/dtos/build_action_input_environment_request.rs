use std::collections::HashMap;

/// Request DTO for the
/// `BuildActionInputEnvironmentPort`
/// inbound port.
pub struct BuildActionInputEnvironmentRequest<'a> {
    /// Environment the action runs with.
    pub env: &'a HashMap<String, String>,
    /// Inputs the action was called with.
    pub inputs: &'a HashMap<String, String>,
    /// Directory the action was copied to inside the container.
    pub action_path: &'a str,
}
