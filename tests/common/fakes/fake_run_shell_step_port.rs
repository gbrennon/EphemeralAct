#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ephact::{
    application::{
        dtos::RunShellStepRequest,
        ports::{outbound::ExecResult, outbound::run_shell_step_port::RunShellStepPort},
    },
    domain::{errors::StepError, workflow::Step},
};

/// Answers every shell step with a prepared result, recording the steps and
/// environments it received. Shares its recordings across clones.
#[derive(Clone)]
pub struct FakeRunShellStepPort {
    result: Result<ExecResult, (String, String, String)>,
    steps: Rc<RefCell<Vec<Step>>>,
    environments: Rc<RefCell<Vec<HashMap<String, String>>>>,
}

impl FakeRunShellStepPort {
    pub fn returning(result: ExecResult) -> Self {
        Self {
            result: Ok(result),
            steps: Rc::new(RefCell::new(Vec::new())),
            environments: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn failing(error: StepError) -> Self {
        Self {
            result: Err((error.message, error.stdout, error.stderr)),
            steps: Rc::new(RefCell::new(Vec::new())),
            environments: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn steps(&self) -> Vec<Step> {
        self.steps.borrow().clone()
    }

    pub fn environments(&self) -> Vec<HashMap<String, String>> {
        self.environments.borrow().clone()
    }
}

impl RunShellStepPort for FakeRunShellStepPort {
    fn execute(&self, request: RunShellStepRequest<'_>) -> Result<ExecResult, StepError> {
        self.steps.borrow_mut().push(request.step.clone());
        self.environments.borrow_mut().push(request.env.clone());
        match &self.result {
            Ok(result) => Ok(result.clone()),
            Err((message, stdout, stderr)) => Err(StepError {
                message: message.clone(),
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            }),
        }
    }
}
