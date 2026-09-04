use crate::application::commands::ExecuteStepCommand;
use crate::application::ports::inbound::execute_step_port::ExecuteStepPort;
use crate::{
    application::dtos::{ExecuteStepRequest, ExecutedStep},
    domain::errors::StepError,
};

/// Infrastructure command handler that processes `ExecuteStepCommand`.
pub struct StepCommandHandler {
    executor: Box<dyn ExecuteStepPort>,
}

impl StepCommandHandler {
    pub fn new(executor: Box<dyn ExecuteStepPort>) -> Self {
        Self { executor }
    }

    pub fn handle(&self, cmd: ExecuteStepCommand) -> Result<ExecutedStep, StepError> {
        let req = ExecuteStepRequest {
            step: &cmd.step,
            context: &cmd.context,
            container: cmd.container,
            repo_path: &cmd.repo_path,
            env: &cmd.env,
        };
        self.executor.execute(req)
    }
}
