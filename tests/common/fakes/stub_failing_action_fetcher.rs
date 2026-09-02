#![allow(dead_code)]
use std::path::PathBuf;

use ephemeral_act::core::{
    errors::ActionError, ports::outbound::ActionFetcherPort, value_objects::RemoteActionReference,
};

/// Fails every fetch, standing in for an unreachable forge.
pub struct StubFailingActionFetcher;

impl ActionFetcherPort for StubFailingActionFetcher {
    fn fetch(&self, reference: &RemoteActionReference) -> Result<PathBuf, ActionError> {
        Err(ActionError::FetchFailed(format!(
            "{} is unreachable",
            reference.clone_url()
        )))
    }
}
