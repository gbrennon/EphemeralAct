use std::{collections::HashMap, path::Path};

use crate::{
    application::{
        dtos::ExecuteActionRequest,
        ports::outbound::execute_nested_action_port::ExecuteNestedActionPort,
    },
    domain::workflow::Step,
};

/// Request DTO for the
/// [`RunCompositeActionPort`](crate::application::ports::outbound::run_composite_action_port::RunCompositeActionPort)
/// inbound port.
pub struct RunCompositeActionRequest<'a> {
    /// Steps the composite action declared.
    pub steps: &'a [Step],
    /// Inputs the action was called with.
    pub inputs: &'a HashMap<String, String>,
    /// Directory holding the action on the host.
    pub action_dir: &'a Path,
    /// The action execution these steps belong to.
    pub action_request: &'a ExecuteActionRequest,
    /// Executor used for a nested `uses:` step.
    pub nested_executor: &'a dyn ExecuteNestedActionPort,
}
