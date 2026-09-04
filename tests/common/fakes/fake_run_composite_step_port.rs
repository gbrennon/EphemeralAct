#![allow(dead_code)]
use parking_lot::Mutex;
use std::sync::Arc;

use ephact::{
    application::dtos::{ExecResult, RunCompositeStepRequest},
    domain::{errors::StepError, workflow::Step},
    infrastructure::steps::run_composite_step_port::RunCompositeStepPort,
};

/// Answers each composite step with the next queued result, recording the
/// steps it was asked to run.
#[derive(Clone, Default)]
pub struct FakeRunCompositeStepPort {
    results: Arc<Mutex<Vec<ExecResult>>>,
    failure: Option<(String, String, String)>,
    steps: Arc<Mutex<Vec<Step>>>,
}

impl FakeRunCompositeStepPort {
    pub fn queueing(results: Vec<ExecResult>) -> Self {
        Self {
            results: Arc::new(Mutex::new(results)),
            failure: None,
            steps: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn failing(error: StepError) -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            failure: Some((error.message, error.stdout, error.stderr)),
            steps: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn steps(&self) -> Vec<Step> {
        self.steps.lock().clone()
    }
}

impl RunCompositeStepPort for FakeRunCompositeStepPort {
    fn execute(&self, request: RunCompositeStepRequest<'_>) -> Result<ExecResult, StepError> {
        self.steps.lock().push(request.step.clone());

        if let Some((message, stdout, stderr)) = &self.failure {
            return Err(StepError {
                message: message.clone(),
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            });
        }

        let mut queued = self.results.lock();
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
