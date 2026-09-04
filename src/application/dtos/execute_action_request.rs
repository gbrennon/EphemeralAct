use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use crate::application::ports::outbound::container_port::ContainerPort;
use crate::domain::expression::EvalContext;
use crate::domain::workflow::Step;

#[derive(Clone)]
pub struct ExecuteActionRequest {
    pub action_ref: String,

    pub step: Step,

    pub repo_path: PathBuf,

    pub env: HashMap<String, String>,

    pub context: EvalContext,

    pub container: Arc<dyn ContainerPort>,
}

impl fmt::Debug for ExecuteActionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecuteActionRequest")
            .field("action_ref", &self.action_ref)
            .field("repo_path", &self.repo_path)
            .finish_non_exhaustive()
    }
}
