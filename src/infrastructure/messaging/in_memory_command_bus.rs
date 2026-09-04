use std::error::Error;

use crate::application::commands::{
    ExecuteActionCommand, ExecuteJobCommand, ExecuteStepCommand, ExecuteWorkflowCommand,
};
use crate::{
    application::{
        dtos::{ExecuteActionResponse, ExecutedStep, JobExecution, WorkflowExecution},
        ports::outbound::CommandBusPort,
    },
    domain::errors::StepError,
    infrastructure::{
        actions::ActionCommandHandler, jobs::JobCommandHandler, steps::StepCommandHandler,
        workflows::WorkflowCommandHandler,
    },
};

/// In-memory implementation of the CommandBusPort.
///
/// Routes commands (intentions of something to happen in the future)
/// to their corresponding infrastructure command handlers.
pub struct InMemoryCommandBus {
    workflow_handler: Box<WorkflowCommandHandler>,
    job_handler: Box<JobCommandHandler>,
    step_handler: Box<StepCommandHandler>,
    action_handler: Box<ActionCommandHandler>,
}

impl InMemoryCommandBus {
    pub fn new(
        workflow_handler: Box<WorkflowCommandHandler>,
        job_handler: Box<JobCommandHandler>,
        step_handler: Box<StepCommandHandler>,
        action_handler: Box<ActionCommandHandler>,
    ) -> Self {
        Self {
            workflow_handler,
            job_handler,
            step_handler,
            action_handler,
        }
    }
}

impl CommandBusPort for InMemoryCommandBus {
    fn dispatch_workflow(
        &self,
        cmd: ExecuteWorkflowCommand,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        self.workflow_handler.handle(cmd)
    }

    fn dispatch_job(&self, cmd: ExecuteJobCommand) -> Result<JobExecution, Box<dyn Error>> {
        self.job_handler.handle(cmd)
    }

    fn dispatch_step(&self, cmd: ExecuteStepCommand) -> Result<ExecutedStep, StepError> {
        self.step_handler.handle(cmd)
    }

    fn dispatch_action(
        &self,
        cmd: ExecuteActionCommand,
    ) -> Result<ExecuteActionResponse, StepError> {
        self.action_handler.handle(cmd)
    }
}
