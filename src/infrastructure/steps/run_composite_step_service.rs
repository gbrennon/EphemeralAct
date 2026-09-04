use std::sync::Arc;

use crate::{
    application::{
        commands::ExecuteActionCommand,
        dtos::{ExecResult, RunCompositeStepRequest, RunShellStepRequest},
        ports::outbound::{
            command_bus_port::CommandBusPort, run_shell_step_port::RunShellStepPort,
        },
    },
    domain::errors::StepError,
    infrastructure::steps::run_composite_step_port::RunCompositeStepPort,
};

/// Runs one step of a composite action: shell steps go straight to the shell
/// runner, while steps referencing another action are published as an
/// [`ExecuteActionCommand`] so the action command handler executes them.
pub struct RunCompositeStepService {
    shell_runner: Box<dyn RunShellStepPort>,
    command_bus: Arc<dyn CommandBusPort>,
}

impl RunCompositeStepService {
    pub fn new(
        shell_runner: Box<dyn RunShellStepPort>,
        command_bus: Arc<dyn CommandBusPort>,
    ) -> Self {
        Self {
            shell_runner,
            command_bus,
        }
    }
}

impl RunCompositeStepPort for RunCompositeStepService {
    fn execute(&self, request: RunCompositeStepRequest<'_>) -> Result<ExecResult, StepError> {
        let action_request = request.action_request;

        match request.step.uses() {
            Some(nested) => self
                .command_bus
                .dispatch_action(ExecuteActionCommand::new(
                    nested.to_string(),
                    request.step.clone(),
                    action_request.repo_path.clone(),
                    action_request.env.clone(),
                    request.context.clone(),
                    action_request.container.clone(),
                ))
                .map(|response| ExecResult {
                    exit_code: response.exit_code,
                    stdout: response.stdout,
                    stderr: response.stderr,
                }),
            None => {
                let mut action_env = action_request.env.clone();
                action_env.insert(
                    "GITHUB_ACTION_PATH".into(),
                    request.action_dir.display().to_string(),
                );
                self.shell_runner.execute(RunShellStepRequest {
                    step: request.step,
                    container: action_request.container.as_ref(),
                    env: &action_env,
                })
            }
        }
    }
}
