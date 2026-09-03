use std::path::PathBuf;

use crate::application::dtos::DetectWorkflowFileRequest;

/// Inbound port for detecting which workflow a repository runs by default.
pub trait DetectWorkflowFilePort {
    /// Returns the first workflow file of the repository's platform directory.
    fn execute(
        &self,
        request: DetectWorkflowFileRequest<'_>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>>;
}
