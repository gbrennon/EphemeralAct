use std::{collections::HashMap, path::Path, sync::Arc};

use crate::application::ports::outbound::container_port::ContainerPort;
use crate::domain::expression::EvalContext;
use crate::domain::workflow::Step;

pub struct ExecuteStepRequest<'a> {
    pub step: &'a Step,

    pub context: &'a EvalContext,

    pub container: Arc<dyn ContainerPort>,

    pub repo_path: &'a Path,

    pub env: &'a HashMap<String, String>,
}
