use std::error::Error;

use crate::application::commands::{
    ExecuteActionCommand, ExecuteJobCommand, ExecuteStepCommand, ExecuteWorkflowCommand,
};
use crate::{
    application::dtos::{ExecuteActionResponse, ExecutedStep, JobExecution, WorkflowExecution},
    domain::errors::StepError,
};

/// Outbound port representing the command bus.
///
/// Dispatches commands (intentions of something to happen in the future)
/// to their corresponding command handlers in the infrastructure layer.
pub trait CommandBusPort: Send + Sync {
    /// Dispatches an [`ExecuteWorkflowCommand`] to the workflow command handler.
    fn dispatch_workflow(
        &self,
        cmd: ExecuteWorkflowCommand,
    ) -> Result<WorkflowExecution, Box<dyn Error>>;

    /// Dispatches an [`ExecuteJobCommand`] to the job command handler.
    fn dispatch_job(&self, cmd: ExecuteJobCommand) -> Result<JobExecution, Box<dyn Error>>;
    /// Dispatches an [`ExecuteStepCommand`] to the step command handler.
    fn dispatch_step(&self, cmd: ExecuteStepCommand) -> Result<ExecutedStep, StepError>;

    /// Dispatches an [`ExecuteActionCommand`] to the action command handler.
    fn dispatch_action(
        &self,
        cmd: ExecuteActionCommand,
    ) -> Result<ExecuteActionResponse, StepError>;
}
