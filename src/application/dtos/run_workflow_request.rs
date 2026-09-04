use crate::domain::{Repository, value_objects::act_run_config::ActRunConfig};

/// Request DTO for executing a single workflow in a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunWorkflowRequest {
    pub config: ActRunConfig,
    pub repository: Repository,
}

impl RunWorkflowRequest {
    pub fn new(config: ActRunConfig, repository: Repository) -> Self {
        Self { config, repository }
    }
}
