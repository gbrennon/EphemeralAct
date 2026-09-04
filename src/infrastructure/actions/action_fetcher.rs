use std::path::PathBuf;

use crate::domain::{errors::ActionError, value_objects::RemoteActionReference};

/// Outbound port for retrieving a remote action's source tree.
///
/// Adapters fetch the referenced repository at the requested revision from
/// whatever forge hosts it and return a directory on the host that contains the
/// checked-out tree. Implementations MAY cache: repeated requests for the same
/// reference SHOULD return the same directory without fetching again.
pub trait ActionFetcherPort: Send + Sync {
    /// Returns a local directory holding the repository the reference points
    /// at, checked out at its revision.
    ///
    /// # Errors
    ///
    /// Returns [`ActionError::FetchFailed`] when the repository or revision
    /// cannot be retrieved.
    fn fetch(&self, reference: &RemoteActionReference) -> Result<PathBuf, ActionError>;
    fn clone_box(&self) -> Box<dyn ActionFetcherPort>;
}

impl Clone for Box<dyn ActionFetcherPort> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
