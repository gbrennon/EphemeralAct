use crate::application::ports::outbound::FileEntry;

/// Response DTO for the
/// [`CollectActionFilesPort`](crate::application::ports::outbound::collect_action_files_port::CollectActionFilesPort)
/// inbound port.
#[derive(Debug)]
pub struct CollectActionFilesResponse {
    /// Files making up the action, with paths relative to its directory.
    pub files: Vec<FileEntry>,
}
