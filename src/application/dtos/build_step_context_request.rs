use std::collections::HashMap;

use crate::domain::expression::EvalContext;

/// Request DTO for the
/// [`BuildStepContextPort`](crate::application::ports::inbound::build_step_context_port::BuildStepContextPort)
/// inbound port.
pub struct BuildStepContextRequest<'a> {
    /// Context of the run the step belongs to.
    pub context: &'a EvalContext,
    /// Environment the step will run with.
    pub env: &'a HashMap<String, String>,
}
