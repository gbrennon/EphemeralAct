use crate::application::commands::ExecuteActionCommand;
use crate::application::ports::inbound::execute_action_port::ExecuteActionPort;
use crate::{
    application::dtos::{ExecuteActionRequest, ExecuteActionResponse},
    domain::errors::StepError,
};

/// Infrastructure command handler that processes `ExecuteActionCommand`.
pub struct ActionCommandHandler {
    executor: Box<dyn ExecuteActionPort>,
}

impl ActionCommandHandler {
    pub fn new(executor: Box<dyn ExecuteActionPort>) -> Self {
        Self { executor }
    }

    pub fn handle(&self, cmd: ExecuteActionCommand) -> Result<ExecuteActionResponse, StepError> {
        let req = ExecuteActionRequest {
            action_ref: cmd.action_ref,
            step: cmd.step,
            repo_path: cmd.repo_path,
            env: cmd.env,
            context: cmd.context,
            container: cmd.container,
        };
        self.executor.execute(req)
    }
}
