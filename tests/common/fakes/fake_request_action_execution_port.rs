#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephact::{
    application::{
        dtos::{ExecuteActionRequest, ExecuteActionResponse},
        ports::inbound::request_action_execution_port::RequestActionExecutionPort,
    },
    domain::errors::StepError,
};

/// Answers every action request with a prepared outcome, recording the
/// requests it received. Shares its recordings across clones.
#[derive(Clone)]
pub struct FakeRequestActionExecutionPort {
    result: Result<ExecuteActionResponse, (String, String, String)>,
    requests: Rc<RefCell<Vec<ExecuteActionRequest>>>,
}

impl FakeRequestActionExecutionPort {
    pub fn returning(response: ExecuteActionResponse) -> Self {
        Self {
            result: Ok(response),
            requests: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn failing(error: StepError) -> Self {
        Self {
            result: Err((error.message, error.stdout, error.stderr)),
            requests: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn requests(&self) -> Vec<ExecuteActionRequest> {
        self.requests.borrow().clone()
    }
}

impl RequestActionExecutionPort for FakeRequestActionExecutionPort {
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError> {
        self.requests.borrow_mut().push(request);
        match &self.result {
            Ok(response) => Ok(response.clone()),
            Err((message, stdout, stderr)) => Err(StepError {
                message: message.clone(),
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            }),
        }
    }
}
