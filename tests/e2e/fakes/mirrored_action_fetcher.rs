use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ephemeral_act::core::{
    errors::ActionError, ports::outbound::ActionFetcherPort, value_objects::RemoteActionReference,
};

/// Action fetcher that resolves every remote reference to one checkout already
/// present on disk, recording what the application asked it to fetch.
#[derive(Clone)]
pub struct MirroredActionFetcher {
    action_directory: PathBuf,
    fetched: Rc<RefCell<Vec<RemoteActionReference>>>,
}

impl MirroredActionFetcher {
    pub fn mirroring(action_directory: PathBuf) -> Self {
        Self {
            action_directory,
            fetched: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn fetched(&self) -> Vec<RemoteActionReference> {
        self.fetched.borrow().clone()
    }
}

impl ActionFetcherPort for MirroredActionFetcher {
    fn fetch(&self, reference: &RemoteActionReference) -> Result<PathBuf, ActionError> {
        self.fetched.borrow_mut().push(reference.clone());
        Ok(self.action_directory.clone())
    }
}
