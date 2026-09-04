#![allow(dead_code)]
use parking_lot::Mutex;
use std::{path::PathBuf, sync::Arc};

use ephact::{
    domain::{errors::ActionError, value_objects::RemoteActionReference},
    infrastructure::actions::ActionFetcherPort,
};

/// Resolves every remote reference to one prepared directory on disk, standing
/// in for a successful clone.
#[derive(Clone)]
pub struct FakeActionFetcher {
    action_dir: PathBuf,
    pub fetched: Arc<Mutex<Vec<RemoteActionReference>>>,
}

impl FakeActionFetcher {
    pub fn returning(action_dir: PathBuf) -> Self {
        Self {
            action_dir,
            fetched: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ActionFetcherPort for FakeActionFetcher {
    fn fetch(&self, reference: &RemoteActionReference) -> Result<PathBuf, ActionError> {
        self.fetched.lock().push(reference.clone());
        Ok(self.action_dir.clone())
    }

    fn clone_box(&self) -> Box<dyn ActionFetcherPort> {
        Box::new(self.clone())
    }
}
