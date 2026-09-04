use crate::application::dtos::ExecResult;
use crate::application::dtos::RunShellStepRequest;
use crate::application::ports::outbound::run_shell_step_port::RunShellStepPort;
use crate::domain::errors::StepError;
use crate::domain::value_objects::ShellCommand;

/// Service that runs a step's shell script inside the container it was given.
pub struct RunShellStepService;

impl RunShellStepService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RunShellStepService {
    fn default() -> Self {
        Self::new()
    }
}

impl RunShellStepPort for RunShellStepService {
    fn execute(&self, request: RunShellStepRequest<'_>) -> Result<ExecResult, StepError> {
        let command = ShellCommand::for_step(request.step, request.env)
            .ok_or_else(|| StepError::new("step has neither `run` nor `uses` defined"))?;

        request
            .container
            .exec(command.argv(), command.working_directory(), command.env())
            .map_err(|error| StepError::new(format!("{error:?}")))
    }
}
