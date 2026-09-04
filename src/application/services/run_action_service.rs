use std::sync::Arc;

use crate::application::commands::ExecuteActionCommand;
use crate::{
    application::{
        dtos::{ExecuteActionResponse, RunActionRequest},
        ports::{inbound::RunActionPort, outbound::CommandBusPort},
    },
    domain::errors::StepError,
};

/// Application service implementing the entrypoint to run an action.
///
/// Depends only on outbound ports (`CommandBusPort`).
pub struct RunActionService {
    command_bus: Arc<dyn CommandBusPort>,
}

impl RunActionService {
    pub fn new(command_bus: Arc<dyn CommandBusPort>) -> Self {
        Self { command_bus }
    }
}

impl RunActionPort for RunActionService {
    fn execute(&self, request: RunActionRequest) -> Result<ExecuteActionResponse, StepError> {
        let cmd = ExecuteActionCommand::new(
            request.action_ref,
            request.step,
            request.repo_path,
            request.env,
            request.context,
            request.container,
        );
        self.command_bus.dispatch_action(cmd)
    }
}
