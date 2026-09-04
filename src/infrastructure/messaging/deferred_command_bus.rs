use std::{error::Error, sync::OnceLock};

use crate::application::commands::{
    ExecuteActionCommand, ExecuteJobCommand, ExecuteStepCommand, ExecuteWorkflowCommand,
};
use crate::{
    application::{
        dtos::{ExecuteActionResponse, ExecutedStep, JobExecution, WorkflowExecution},
        ports::outbound::CommandBusPort,
    },
    domain::errors::StepError,
};

/// Command bus whose target is bound after construction.
///
/// The command graph is cyclic by design: a coordination service publishes a
/// command that a handler turns into a call on the next coordination service.
/// Handing every service this proxy first, then binding the assembled bus,
/// closes that cycle without any service holding a handler.
#[derive(Default)]
pub struct DeferredCommandBus {
    bus: OnceLock<Box<dyn CommandBusPort>>,
}

impl DeferredCommandBus {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bus: OnceLock::new(),
        }
    }

    /// Binds the bus every dispatch is forwarded to.
    ///
    /// # Panics
    ///
    /// Panics when a bus is already bound: rebinding would silently reroute
    /// commands already in flight.
    pub fn bind(&self, bus: Box<dyn CommandBusPort>) {
        assert!(self.bus.set(bus).is_ok(), "command bus already bound");
    }
    fn bound(&self) -> Option<&dyn CommandBusPort> {
        self.bus.get().map(AsRef::as_ref)
    }

    fn unbound() -> StepError {
        StepError::new("command bus used before it was bound".to_string())
    }
}
impl CommandBusPort for DeferredCommandBus {
    fn dispatch_workflow(
        &self,
        cmd: ExecuteWorkflowCommand,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        self.bound()
            .ok_or_else(|| Self::unbound().message)?
            .dispatch_workflow(cmd)
    }

    fn dispatch_job(&self, cmd: ExecuteJobCommand) -> Result<JobExecution, Box<dyn Error>> {
        self.bound()
            .ok_or_else(|| Self::unbound().message)?
            .dispatch_job(cmd)
    }

    fn dispatch_step(&self, cmd: ExecuteStepCommand) -> Result<ExecutedStep, StepError> {
        self.bound().ok_or_else(Self::unbound)?.dispatch_step(cmd)
    }

    fn dispatch_action(
        &self,
        cmd: ExecuteActionCommand,
    ) -> Result<ExecuteActionResponse, StepError> {
        self.bound().ok_or_else(Self::unbound)?.dispatch_action(cmd)
    }
}
