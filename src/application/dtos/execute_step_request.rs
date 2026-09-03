use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{
    application::ports::outbound::ContainerPort,
    domain::{expression::EvalContext, workflow::Step},
};

/// Request DTO for the
/// [`ExecuteStepPort`](crate::application::ports::outbound::execute_step_port::ExecuteStepPort)
/// inbound port.
pub struct ExecuteStepRequest<'a> {
    /// Step to execute, before its expressions are resolved.
    pub step: &'a Step,
    /// Context the step's expressions are resolved against.
    pub context: &'a EvalContext,
    /// Container the step runs in.
    pub container: Arc<dyn ContainerPort>,
    /// Repository directory the run executes against.
    pub repo_path: &'a Path,
    /// Environment the step runs with.
    pub env: &'a HashMap<String, String>,
}
