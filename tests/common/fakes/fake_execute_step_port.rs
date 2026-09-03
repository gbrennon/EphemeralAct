#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ephact::{
    application::{
        dtos::{ExecuteActionResponse, ExecuteStepRequest, ExecutedStep},
        ports::outbound::execute_step_port::ExecuteStepPort,
    },
    domain::{errors::StepError, expression::EvalContext, workflow::Step},
};

/// Answers each step with the next queued exit code, recording the step, the
/// environment and the context it was asked to run with.
#[derive(Clone, Default)]
pub struct FakeExecuteStepPort {
    exit_codes: Rc<RefCell<Vec<i64>>>,
    steps: Rc<RefCell<Vec<Step>>>,
    environments: Rc<RefCell<Vec<HashMap<String, String>>>>,
    contexts: Rc<RefCell<Vec<EvalContext>>>,
}

impl FakeExecuteStepPort {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queueing(exit_codes: Vec<i64>) -> Self {
        Self {
            exit_codes: Rc::new(RefCell::new(exit_codes)),
            ..Self::default()
        }
    }

    pub fn steps(&self) -> Vec<Step> {
        self.steps.borrow().clone()
    }

    pub fn environments(&self) -> Vec<HashMap<String, String>> {
        self.environments.borrow().clone()
    }

    pub fn contexts(&self) -> Vec<EvalContext> {
        self.contexts.borrow().clone()
    }
}

impl ExecuteStepPort for FakeExecuteStepPort {
    fn execute(&self, request: ExecuteStepRequest<'_>) -> Result<ExecutedStep, StepError> {
        self.steps.borrow_mut().push(request.step.clone());
        self.environments.borrow_mut().push(request.env.clone());
        self.contexts.borrow_mut().push(request.context.clone());

        let mut queued = self.exit_codes.borrow_mut();
        let exit_code = if queued.is_empty() {
            0
        } else {
            queued.remove(0)
        };

        Ok(ExecutedStep {
            step: request.step.clone(),
            response: ExecuteActionResponse {
                exit_code,
                stdout: String::new(),
                stderr: String::new(),
            },
        })
    }
}
