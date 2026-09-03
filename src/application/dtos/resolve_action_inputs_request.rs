use crate::domain::workflow::{ActionDefinition, Step};

/// Request DTO for the
/// [`ResolveActionInputsPort`](crate::application::ports::outbound::resolve_action_inputs_port::ResolveActionInputsPort)
/// inbound port.
pub struct ResolveActionInputsRequest<'a> {
    /// Definition declaring the action's inputs and their defaults.
    pub definition: &'a ActionDefinition,
    /// Step that referenced the action, for its `with:` values.
    pub step: &'a Step,
}
