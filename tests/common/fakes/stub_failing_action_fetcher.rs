#![allow(dead_code)]
use std::path::PathBuf;

use ephact::{
    domain::{errors::ActionError, value_objects::RemoteActionReference},
    infrastructure::actions::ActionFetcherPort,
};

/// Fails every fetch, standing in for an unreachable forge.
#[derive(Clone)]
pub struct StubFailingActionFetcher;

impl ActionFetcherPort for StubFailingActionFetcher {
    fn fetch(&self, reference: &RemoteActionReference) -> Result<PathBuf, ActionError> {
        Err(ActionError::FetchFailed(format!(
            "{} is unreachable",
            reference.clone_url()
        )))
    }

    fn clone_box(&self) -> Box<dyn ActionFetcherPort> {
        Box::new(self.clone())
    }
}
