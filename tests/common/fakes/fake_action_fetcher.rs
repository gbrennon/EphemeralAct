#![allow(dead_code)]
use std::{cell::RefCell, path::PathBuf};

use ephact::{
    application::ports::outbound::ActionFetcherPort,
    domain::{errors::ActionError, value_objects::RemoteActionReference},
};

/// Resolves every remote reference to one prepared directory on disk, standing
/// in for a successful clone.
pub struct FakeActionFetcher {
    action_dir: PathBuf,
    pub fetched: RefCell<Vec<RemoteActionReference>>,
}

impl FakeActionFetcher {
    pub fn returning(action_dir: PathBuf) -> Self {
        Self {
            action_dir,
            fetched: RefCell::new(Vec::new()),
        }
    }
}

impl ActionFetcherPort for FakeActionFetcher {
    fn fetch(&self, reference: &RemoteActionReference) -> Result<PathBuf, ActionError> {
        self.fetched.borrow_mut().push(reference.clone());
        Ok(self.action_dir.clone())
    }
}
