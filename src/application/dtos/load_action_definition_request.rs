use std::path::Path;

/// Request DTO for the
/// [`LoadActionDefinitionPort`](crate::application::ports::outbound::load_action_definition_port::LoadActionDefinitionPort)
/// inbound port.
pub struct LoadActionDefinitionRequest<'a> {
    /// Directory holding the action's `action.yml`.
    pub action_dir: &'a Path,
}
