use std::collections::HashMap;

/// Request DTO for the
/// [`PrefixStepPathPort`](crate::application::ports::outbound::prefix_step_path_port::PrefixStepPathPort)
/// inbound port.
pub struct PrefixStepPathRequest<'a> {
    /// Environment whose `PATH` is prefixed.
    pub env: &'a HashMap<String, String>,
    /// Directories earlier steps exported through `GITHUB_PATH`.
    pub path_additions: &'a [String],
}
