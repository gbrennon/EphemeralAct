use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use crate::core::{expression::EvalContext, ports::outbound::ContainerPort, workflow::Step};

/// Request DTO for the
/// [`ExecuteActionPort`](crate::core::ports::inbound::execute_action_port::ExecuteActionPort)
/// inbound port.
///
/// Carries everything an action needs to run: the raw `uses:` reference, the
/// step that declared it (for `with:` inputs), the checked-out repository, the
/// job environment, the expression context the action's own expressions are
/// evaluated against, and the container the action runs inside.
#[derive(Clone)]
pub struct ExecuteActionRequest {
    /// The `uses:` value, already interpolated.
    pub action_ref: String,
    /// The step that referenced the action.
    pub step: Step,
    /// Root of the repository under test on the host.
    pub repo_path: PathBuf,
    /// Environment variables visible to the action.
    pub env: HashMap<String, String>,
    /// Context used to evaluate expressions inside the action.
    pub context: EvalContext,
    /// Container the action executes in.
    pub container: Arc<dyn ContainerPort>,
}

impl fmt::Debug for ExecuteActionRequest {
    /// Describes the request without the container handle, which has no
    /// meaningful debug representation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecuteActionRequest")
            .field("action_ref", &self.action_ref)
            .field("repo_path", &self.repo_path)
            .finish_non_exhaustive()
    }
}
