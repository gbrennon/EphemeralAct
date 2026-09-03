use std::path::Path;

/// Request DTO for the
/// [`ResolveActionDirectoryPort`](crate::application::ports::inbound::resolve_action_directory_port::ResolveActionDirectoryPort)
/// inbound port.
pub struct ResolveActionDirectoryRequest<'a> {
    /// The `uses:` value naming the action.
    pub action_ref: &'a str,
    /// Root of the repository under test.
    pub repo_path: &'a Path,
}
