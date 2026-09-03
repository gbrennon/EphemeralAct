#![allow(dead_code)]
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ephact::{
    application::{
        dtos::FetchRemoteActionRequest,
        ports::outbound::fetch_remote_action_port::FetchRemoteActionPort,
    },
    domain::{errors::ActionError, value_objects::RemoteActionReference},
};

/// Resolves every remote reference to a prepared directory, or fails.
#[derive(Clone)]
pub struct FakeFetchRemoteActionPort {
    result: Result<PathBuf, String>,
    fetched: Rc<RefCell<Vec<RemoteActionReference>>>,
}

impl FakeFetchRemoteActionPort {
    pub fn returning(directory: PathBuf) -> Self {
        Self {
            result: Ok(directory),
            fetched: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            fetched: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn fetched(&self) -> Vec<RemoteActionReference> {
        self.fetched.borrow().clone()
    }
}

impl FetchRemoteActionPort for FakeFetchRemoteActionPort {
    fn execute(&self, request: FetchRemoteActionRequest<'_>) -> Result<PathBuf, ActionError> {
        self.fetched.borrow_mut().push(request.reference.clone());
        self.result.clone().map_err(ActionError::FetchFailed)
    }
}
