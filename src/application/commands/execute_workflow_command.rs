use crate::domain::{
    entities::repository::Repository, value_objects::act_run_config::ActRunConfig,
};

/// Command representing the intention to execute a workflow.
///
/// Carries the already-resolved workflow definition content and domain objects only:
/// no filesystem paths, handles, or infrastructure references are exposed here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecuteWorkflowCommand {
    pub workflow_content: String,
    pub config: ActRunConfig,
    pub repository: Repository,
}

impl ExecuteWorkflowCommand {
    pub fn new(workflow_content: String, config: ActRunConfig, repository: Repository) -> Self {
        Self {
            workflow_content,
            config,
            repository,
        }
    }
}
