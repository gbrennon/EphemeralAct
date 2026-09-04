#![allow(dead_code)]
use ephact::infrastructure::actions::fetch_remote_action_port::FetchRemoteActionPort;
use parking_lot::Mutex;
use std::{path::PathBuf, sync::Arc};

use ephact::{
    application::dtos::FetchRemoteActionRequest,
    domain::{errors::ActionError, value_objects::RemoteActionReference},
};

/// Resolves every remote reference to a prepared directory, or fails.
#[derive(Clone)]
pub struct FakeFetchRemoteActionPort {
    result: Result<PathBuf, String>,
    fetched: Arc<Mutex<Vec<RemoteActionReference>>>,
}

impl FakeFetchRemoteActionPort {
    pub fn returning(directory: PathBuf) -> Self {
        Self {
            result: Ok(directory),
            fetched: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            fetched: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn fetched(&self) -> Vec<RemoteActionReference> {
        self.fetched.lock().clone()
    }
}

impl FetchRemoteActionPort for FakeFetchRemoteActionPort {
    fn execute(&self, request: FetchRemoteActionRequest<'_>) -> Result<PathBuf, ActionError> {
        self.fetched.lock().push(request.reference.clone());
        self.result.clone().map_err(ActionError::FetchFailed)
    }
}
