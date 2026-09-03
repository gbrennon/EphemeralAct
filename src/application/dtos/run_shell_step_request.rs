use std::collections::HashMap;

use crate::{application::ports::outbound::ContainerPort, domain::workflow::Step};

/// Request DTO for the
/// [`RunShellStepPort`](crate::application::ports::inbound::run_shell_step_port::RunShellStepPort)
/// inbound port.
pub struct RunShellStepRequest<'a> {
    /// Step whose `run:` script is executed.
    pub step: &'a Step,
    /// Container the script runs in.
    pub container: &'a dyn ContainerPort,
    /// Environment the script runs with.
    pub env: &'a HashMap<String, String>,
}
