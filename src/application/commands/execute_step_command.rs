use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use crate::application::ports::outbound::container_port::ContainerPort;
use crate::domain::expression::EvalContext;
use crate::domain::workflow::Step;

/// Command representing the intention to execute one step of a job.
///
/// Published by the job coordination service for every step of the job, and
/// handled by the step command handler.
#[derive(Clone)]
pub struct ExecuteStepCommand {
    pub step: Step,
    pub env: HashMap<String, String>,
    pub context: EvalContext,
    pub container: Arc<dyn ContainerPort>,
    pub repo_path: PathBuf,
}

impl ExecuteStepCommand {
    pub fn new(
        step: Step,
        env: HashMap<String, String>,
        context: EvalContext,
        container: Arc<dyn ContainerPort>,
        repo_path: PathBuf,
    ) -> Self {
        Self {
            step,
            env,
            context,
            container,
            repo_path,
        }
    }
}

impl fmt::Debug for ExecuteStepCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecuteStepCommand")
            .field("uses", &self.step.uses())
            .field("repo_path", &self.repo_path)
            .finish_non_exhaustive()
    }
}
