use std::path::PathBuf;

use crate::{application::dtos::FetchRemoteActionRequest, domain::errors::ActionError};

/// Inbound port for retrieving an action published on a forge.
pub trait FetchRemoteActionPort: Send + Sync {
    /// Fetches the action and returns the directory holding it.
    fn execute(&self, request: FetchRemoteActionRequest<'_>) -> Result<PathBuf, ActionError>;
}
