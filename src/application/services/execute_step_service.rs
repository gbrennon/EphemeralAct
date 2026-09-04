use std::sync::Arc;

use crate::application::commands::ExecuteActionCommand;
use crate::{
    application::{
        dtos::{ExecuteActionResponse, ExecuteStepRequest, ExecutedStep, RunShellStepRequest},
        ports::{
            inbound::execute_step_port::ExecuteStepPort,
            outbound::{command_bus_port::CommandBusPort, run_shell_step_port::RunShellStepPort},
        },
    },
    domain::{errors::StepError, expression::StepInterpolator},
};

/// Application service coordinating the execution of one step.
///
/// Resolves the step's expressions, then either runs it as a shell script
/// through an outbound port or publishes an [`ExecuteActionCommand`] when the
/// step references an action: the action command handler owns that execution.
pub struct ExecuteStepService {
    shell_runner: Box<dyn RunShellStepPort>,
    command_bus: Arc<dyn CommandBusPort>,
}

impl ExecuteStepService {
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

impl ExecuteStepPort for ExecuteStepService {
    fn execute(&self, request: ExecuteStepRequest<'_>) -> Result<ExecutedStep, StepError> {
        let interpolated = StepInterpolator::interpolate(request.step, request.context)
            .map_err(|error| StepError::new(format!("failed to resolve expressions: {error:?}")))?;

        let response = match interpolated.uses() {
            Some(action_ref) => self.command_bus.dispatch_action(ExecuteActionCommand::new(
                action_ref.to_string(),
                interpolated.clone(),
                request.repo_path.to_path_buf(),
                request.env.clone(),
                request.context.clone(),
                request.container,
            ))?,
            None => {
                let result = self.shell_runner.execute(RunShellStepRequest {
                    step: &interpolated,
                    container: request.container.as_ref(),
                    env: request.env,
                })?;
                ExecuteActionResponse {
                    exit_code: result.exit_code,
                    stdout: result.stdout,
                    stderr: result.stderr,
                }
            }
        };

        Ok(ExecutedStep {
            step: interpolated,
            response,
        })
    }
}
