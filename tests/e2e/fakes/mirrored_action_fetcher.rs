use parking_lot::Mutex;
use std::{path::PathBuf, sync::Arc};

use ephact::{
    domain::{errors::ActionError, value_objects::RemoteActionReference},
    infrastructure::actions::ActionFetcherPort,
};

/// Action fetcher that resolves every remote reference to one checkout already
/// present on disk, recording what the application asked it to fetch.
#[derive(Clone)]
pub struct MirroredActionFetcher {
    action_directory: PathBuf,
    fetched: Arc<Mutex<Vec<RemoteActionReference>>>,
}

impl MirroredActionFetcher {
    pub fn mirroring(action_directory: PathBuf) -> Self {
        Self {
            action_directory,
            fetched: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn fetched(&self) -> Vec<RemoteActionReference> {
        self.fetched.lock().clone()
    }
}

impl ActionFetcherPort for MirroredActionFetcher {
    fn fetch(&self, reference: &RemoteActionReference) -> Result<PathBuf, ActionError> {
        self.fetched.lock().push(reference.clone());
        Ok(self.action_directory.clone())
    }

    fn clone_box(&self) -> Box<dyn ActionFetcherPort> {
        Box::new(self.clone())
    }
}
