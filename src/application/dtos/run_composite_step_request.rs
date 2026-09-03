use std::path::Path;

use crate::{
    application::{
        dtos::ExecuteActionRequest,
        ports::outbound::execute_nested_action_port::ExecuteNestedActionPort,
    },
    domain::{expression::EvalContext, workflow::Step},
};

/// Request DTO for the
/// [`RunCompositeStepPort`](crate::application::ports::outbound::run_composite_step_port::RunCompositeStepPort)
/// inbound port.
pub struct RunCompositeStepRequest<'a> {
    /// Step of the composite action, already interpolated.
    pub step: &'a Step,
    /// Directory of the composite action on the host.
    pub action_dir: &'a Path,
    /// The action execution this composite step belongs to.
    pub action_request: &'a ExecuteActionRequest,
    /// Context the composite action's own expressions were resolved against.
    pub context: &'a EvalContext,
    /// Executor used for a nested `uses:` step.
    pub nested_executor: &'a dyn ExecuteNestedActionPort,
}
