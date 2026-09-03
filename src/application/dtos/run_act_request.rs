use crate::domain::{Repository, value_objects::act_run_config::ActRunConfig};

/// Request DTO for the [`RunActPort`](crate::application::ports::inbound::run_act_port::RunActPort)
/// inbound port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunActRequest {
    /// Configuration for the run.
    pub config: ActRunConfig,
    /// Repository to execute the workflow against.
    pub repository: Repository,
}

impl RunActRequest {
    /// Creates a new run request.
    pub fn new(config: ActRunConfig, repository: Repository) -> Self {
        Self { config, repository }
    }
}
