use crate::domain::{Repository, value_objects::act_run_config::ActRunConfig};

/// Request DTO for executing all workflows in a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunAllWorkflowsRequest {
    pub config: ActRunConfig,
    pub repository: Repository,
}

impl RunAllWorkflowsRequest {
    pub fn new(config: ActRunConfig, repository: Repository) -> Self {
        Self { config, repository }
    }
}
