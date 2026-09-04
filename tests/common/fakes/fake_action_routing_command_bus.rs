#![allow(dead_code)]
use std::{
    error::Error,
    sync::{Arc, OnceLock},
};

use ephact::application::commands::{
    ExecuteActionCommand, ExecuteJobCommand, ExecuteStepCommand, ExecuteWorkflowCommand,
};
use ephact::{
    application::{
        dtos::{
            ExecuteActionRequest, ExecuteActionResponse, ExecutedStep, JobExecution,
            WorkflowExecution,
        },
        ports::{inbound::ExecuteActionPort, outbound::CommandBusPort},
    },
    domain::errors::StepError,
};

/// Routes dispatched action commands to a bound action executor, so a
/// composite action nesting another action exercises the real recursion the
/// command bus provides in production. Every other command is rejected.
#[derive(Default)]
pub struct FakeActionRoutingCommandBus {
    executor: OnceLock<Arc<dyn ExecuteActionPort>>,
}

impl FakeActionRoutingCommandBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&self, executor: Arc<dyn ExecuteActionPort>) {
        assert!(
            self.executor.set(executor).is_ok(),
            "executor already bound"
        );
    }
}

impl CommandBusPort for FakeActionRoutingCommandBus {
    fn dispatch_workflow(
        &self,
        _cmd: ExecuteWorkflowCommand,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        Err("workflow commands are not routed by this fake".into())
    }

    fn dispatch_job(&self, _cmd: ExecuteJobCommand) -> Result<JobExecution, Box<dyn Error>> {
        Err("job commands are not routed by this fake".into())
    }

    fn dispatch_step(&self, _cmd: ExecuteStepCommand) -> Result<ExecutedStep, StepError> {
        Err(StepError::new("step commands are not routed by this fake"))
    }

    fn dispatch_action(
        &self,
        cmd: ExecuteActionCommand,
    ) -> Result<ExecuteActionResponse, StepError> {
        let executor = self
            .executor
            .get()
            .ok_or_else(|| StepError::new("no action executor bound"))?;
        executor.execute(ExecuteActionRequest {
            action_ref: cmd.action_ref,
            step: cmd.step,
            repo_path: cmd.repo_path,
            env: cmd.env,
            context: cmd.context,
            container: cmd.container,
        })
    }
}
