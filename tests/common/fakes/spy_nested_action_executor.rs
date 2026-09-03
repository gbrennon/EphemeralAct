#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephact::{
    application::{
        dtos::{ExecuteActionRequest, ExecuteActionResponse},
        ports::outbound::execute_nested_action_port::ExecuteNestedActionPort,
    },
    domain::errors::StepError,
};

/// Nested executor that records the action requests handed to it and answers
/// each with a prepared response.
#[derive(Clone)]
pub struct SpyNestedActionExecutor {
    response: ExecuteActionResponse,
    requests: Rc<RefCell<Vec<ExecuteActionRequest>>>,
}

impl SpyNestedActionExecutor {
    pub fn returning(response: ExecuteActionResponse) -> Self {
        Self {
            response,
            requests: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn requests(&self) -> Vec<ExecuteActionRequest> {
        self.requests.borrow().clone()
    }
}

impl ExecuteNestedActionPort for SpyNestedActionExecutor {
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError> {
        self.requests.borrow_mut().push(request);
        Ok(self.response.clone())
    }
}
