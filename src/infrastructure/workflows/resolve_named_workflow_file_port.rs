use std::path::PathBuf;

use crate::application::dtos::ResolveNamedWorkflowFileRequest;

/// Inbound port for resolving the file of a workflow named by the caller.
pub trait ResolveNamedWorkflowFilePort: Send + Sync {
    /// Resolves the named workflow against the repository root and the
    /// supported platform directories.
    fn execute(
        &self,
        request: ResolveNamedWorkflowFileRequest<'_>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>>;
}
