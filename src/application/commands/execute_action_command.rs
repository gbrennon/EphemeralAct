use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use crate::application::ports::outbound::container_port::ContainerPort;
use crate::domain::expression::EvalContext;
use crate::domain::workflow::Step;

#[derive(Clone)]
pub struct ExecuteActionCommand {
    pub action_ref: String,
    pub step: Step,
    pub repo_path: PathBuf,
    pub env: HashMap<String, String>,
    pub context: EvalContext,
    pub container: Arc<dyn ContainerPort>,
}

impl ExecuteActionCommand {
    pub fn new(
        action_ref: String,
        step: Step,
        repo_path: PathBuf,
        env: HashMap<String, String>,
        context: EvalContext,
        container: Arc<dyn ContainerPort>,
    ) -> Self {
        Self {
            action_ref,
            step,
            repo_path,
            env,
            context,
            container,
        }
    }
}

impl fmt::Debug for ExecuteActionCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecuteActionCommand")
            .field("action_ref", &self.action_ref)
            .field("repo_path", &self.repo_path)
            .finish_non_exhaustive()
    }
}
