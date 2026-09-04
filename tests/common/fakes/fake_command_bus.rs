#![allow(dead_code)]
use parking_lot::Mutex;
use std::{collections::HashMap, error::Error, sync::Arc};

use ephact::application::commands::{
    ExecuteActionCommand, ExecuteJobCommand, ExecuteStepCommand, ExecuteWorkflowCommand,
};
use ephact::{
    application::{
        dtos::{ExecuteActionResponse, ExecutedStep, JobExecution, JobSummary, WorkflowExecution},
        ports::outbound::CommandBusPort,
    },
    domain::errors::StepError,
};

/// Records every dispatched command and answers it with a prepared outcome, so
/// a coordination service can be tested on what it publishes instead of on
/// what the next service does. Shares its recordings across clones.
#[derive(Clone, Default)]
pub struct FakeCommandBus {
    pub dispatched_workflows: Arc<Mutex<Vec<ExecuteWorkflowCommand>>>,
    pub dispatched_jobs: Arc<Mutex<Vec<ExecuteJobCommand>>>,
    pub dispatched_steps: Arc<Mutex<Vec<ExecuteStepCommand>>>,
    pub dispatched_actions: Arc<Mutex<Vec<ExecuteActionCommand>>>,
    workflow_result: Option<WorkflowExecution>,
    action_result: Option<ExecuteActionResponse>,
    failing_jobs: Vec<String>,
    step_exit_codes: Arc<Mutex<Vec<i64>>>,
    step_error: Option<String>,
    job_error: Option<String>,
}

impl FakeCommandBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_workflow_result(mut self, result: WorkflowExecution) -> Self {
        self.workflow_result = Some(result);
        self
    }

    pub fn with_action_result(mut self, result: ExecuteActionResponse) -> Self {
        self.action_result = Some(result);
        self
    }

    /// Answers the named jobs with a failed summary.
    pub fn failing_jobs(mut self, job_ids: Vec<String>) -> Self {
        self.failing_jobs = job_ids;
        self
    }

    /// Answers every job dispatch with an error.
    pub fn failing_job_dispatch(mut self, message: &str) -> Self {
        self.job_error = Some(message.to_string());
        self
    }

    /// Answers every step dispatch with an error.
    pub fn failing_step_dispatch(mut self, message: &str) -> Self {
        self.step_error = Some(message.to_string());
        self
    }

    /// Answers step dispatches with the queued exit codes, in order.
    pub fn queueing_step_exit_codes(self, exit_codes: Vec<i64>) -> Self {
        *self.step_exit_codes.lock() = exit_codes.into_iter().rev().collect();
        self
    }

    /// Environments carried by the dispatched step commands, in order.
    pub fn dispatched_step_environments(&self) -> Vec<HashMap<String, String>> {
        self.dispatched_steps
            .lock()
            .iter()
            .map(|cmd| cmd.env.clone())
            .collect()
    }

    pub fn dispatched_job_ids(&self) -> Vec<String> {
        self.dispatched_jobs
            .lock()
            .iter()
            .map(|cmd| cmd.job_id.clone())
            .collect()
    }

    pub fn dispatched_action_refs(&self) -> Vec<String> {
        self.dispatched_actions
            .lock()
            .iter()
            .map(|cmd| cmd.action_ref.clone())
            .collect()
    }
}

impl CommandBusPort for FakeCommandBus {
    fn dispatch_workflow(
        &self,
        cmd: ExecuteWorkflowCommand,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        self.dispatched_workflows.lock().push(cmd);
        Ok(self.workflow_result.clone().unwrap_or(WorkflowExecution {
            workflow_name: "fake-workflow".into(),
            job_summaries: Vec::new(),
            container_names: vec!["c1".into()],
            success: true,
        }))
    }

    fn dispatch_job(&self, cmd: ExecuteJobCommand) -> Result<JobExecution, Box<dyn Error>> {
        let job_id = cmd.job_id.clone();
        let name = cmd.job.name.clone();
        self.dispatched_jobs.lock().push(cmd);
        if let Some(message) = &self.job_error {
            return Err(message.clone().into());
        }
        Ok(JobExecution {
            job_summary: JobSummary {
                job_id: job_id.clone(),
                name,
                steps: Vec::new(),
                success: !self.failing_jobs.contains(&job_id),
            },
            container_name: format!("container-{job_id}"),
        })
    }

    fn dispatch_step(&self, cmd: ExecuteStepCommand) -> Result<ExecutedStep, StepError> {
        let step = cmd.step.clone();
        self.dispatched_steps.lock().push(cmd);
        if let Some(message) = &self.step_error {
            return Err(StepError::new(message.clone()));
        }
        Ok(ExecutedStep {
            step,
            response: ExecuteActionResponse {
                exit_code: self.step_exit_codes.lock().pop().unwrap_or(0),
                stdout: String::new(),
                stderr: String::new(),
            },
        })
    }

    fn dispatch_action(
        &self,
        cmd: ExecuteActionCommand,
    ) -> Result<ExecuteActionResponse, StepError> {
        self.dispatched_actions.lock().push(cmd);
        Ok(self.action_result.clone().unwrap_or(ExecuteActionResponse {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }))
    }
}
