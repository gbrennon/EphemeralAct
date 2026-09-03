#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephact::{
    application::{
        dtos::{ExecuteActionRequest, ExecuteActionResponse},
        ports::inbound::ExecuteActionPort,
    },
    domain::errors::StepError,
};

/// Action handler that records the references it is asked to run and always
/// reports success. Every clone observes the same recording.
#[derive(Clone, Default)]
pub struct SpyActionHandler {
    requested: Rc<RefCell<Vec<String>>>,
}

impl SpyActionHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn requested(&self) -> Vec<String> {
        self.requested.borrow().clone()
    }
}

impl ExecuteActionPort for SpyActionHandler {
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError> {
        self.requested.borrow_mut().push(request.action_ref.clone());
        Ok(ExecuteActionResponse {
            exit_code: 0,
            stdout: "action ran\n".into(),
            stderr: String::new(),
        })
    }
}
