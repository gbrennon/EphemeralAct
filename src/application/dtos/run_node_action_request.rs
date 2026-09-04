use std::{collections::HashMap, path::Path};

use crate::application::ports::outbound::container_port::ContainerPort;

/// Request DTO for the
/// [`RunNodeActionPort`](crate::application::ports::inbound::run_node_action_port::RunNodeActionPort)
/// outbound port.
pub struct RunNodeActionRequest<'a> {
    /// Directory holding the action on the host.
    pub action_dir: &'a Path,
    /// Entry point the action declared.
    pub entry_point: &'a str,
    /// Inputs the action was called with.
    pub inputs: &'a HashMap<String, String>,
    /// Environment the action runs with.
    pub env: &'a HashMap<String, String>,
    /// Container the action runs in.
    pub container: &'a dyn ContainerPort,
}
