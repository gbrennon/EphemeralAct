use std::path::PathBuf;

use crate::application::dtos::ExecuteActionResponse;

/// Where an action's files live, or why running it needs no work.
#[derive(Debug)]
pub enum ResolvedActionDirectory {
    /// Directory on the host holding the action to run.
    Directory(PathBuf),
    /// The action needs no work; this response is the step's result.
    Skipped(ExecuteActionResponse),
}
