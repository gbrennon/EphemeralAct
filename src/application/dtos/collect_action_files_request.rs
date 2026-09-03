use std::path::Path;

/// Request DTO for the
/// [`CollectActionFilesPort`](crate::application::ports::outbound::collect_action_files_port::CollectActionFilesPort)
/// inbound port.
pub struct CollectActionFilesRequest<'a> {
    /// Directory whose files are collected.
    pub action_dir: &'a Path,
}
