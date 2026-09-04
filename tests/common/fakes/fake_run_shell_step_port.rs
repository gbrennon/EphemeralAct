#![allow(dead_code)]
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};

use ephact::application::dtos::ExecResult;
use ephact::application::dtos::RunShellStepRequest;
use ephact::application::ports::outbound::run_shell_step_port::RunShellStepPort;
use ephact::domain::errors::StepError;
use ephact::domain::workflow::Step;

/// Answers every shell step with a prepared result, recording the steps and
/// environments it received. Shares its recordings across clones.
#[derive(Clone)]
pub struct FakeRunShellStepPort {
    result: Result<ExecResult, (String, String, String)>,
    steps: Arc<Mutex<Vec<Step>>>,
    environments: Arc<Mutex<Vec<HashMap<String, String>>>>,
}

impl FakeRunShellStepPort {
    pub fn returning(result: ExecResult) -> Self {
        Self {
            result: Ok(result),
            steps: Arc::new(Mutex::new(Vec::new())),
            environments: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn failing(error: StepError) -> Self {
        Self {
            result: Err((error.message, error.stdout, error.stderr)),
            steps: Arc::new(Mutex::new(Vec::new())),
            environments: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn steps(&self) -> Vec<Step> {
        self.steps.lock().clone()
    }

    pub fn environments(&self) -> Vec<HashMap<String, String>> {
        self.environments.lock().clone()
    }
}

impl RunShellStepPort for FakeRunShellStepPort {
    fn execute(&self, request: RunShellStepRequest<'_>) -> Result<ExecResult, StepError> {
        self.steps.lock().push(request.step.clone());
        self.environments.lock().push(request.env.clone());
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
