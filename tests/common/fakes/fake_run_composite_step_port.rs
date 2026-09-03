#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephact::{
    application::{
        dtos::RunCompositeStepRequest,
        ports::{outbound::ExecResult, outbound::run_composite_step_port::RunCompositeStepPort},
    },
    domain::{errors::StepError, workflow::Step},
};

/// Answers each composite step with the next queued result, recording the
/// steps it was asked to run.
#[derive(Clone, Default)]
pub struct FakeRunCompositeStepPort {
    results: Rc<RefCell<Vec<ExecResult>>>,
    failure: Option<(String, String, String)>,
    steps: Rc<RefCell<Vec<Step>>>,
}

impl FakeRunCompositeStepPort {
    pub fn queueing(results: Vec<ExecResult>) -> Self {
        Self {
            results: Rc::new(RefCell::new(results)),
            failure: None,
            steps: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn failing(error: StepError) -> Self {
        Self {
            results: Rc::new(RefCell::new(Vec::new())),
            failure: Some((error.message, error.stdout, error.stderr)),
            steps: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn steps(&self) -> Vec<Step> {
        self.steps.borrow().clone()
    }
}

impl RunCompositeStepPort for FakeRunCompositeStepPort {
    fn execute(&self, request: RunCompositeStepRequest<'_>) -> Result<ExecResult, StepError> {
        self.steps.borrow_mut().push(request.step.clone());

        if let Some((message, stdout, stderr)) = &self.failure {
            return Err(StepError {
                message: message.clone(),
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            });
        }

        let mut queued = self.results.borrow_mut();
        if queued.is_empty() {
            return Ok(ExecResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            });
        }
        Ok(queued.remove(0))
    }
}
