use crate::{
    application::{
        dtos::{
            ExecuteActionRequest, ExecuteActionResponse, ExecuteStepRequest, ExecutedStep,
            RunShellStepRequest,
        },
        ports::outbound::{
            execute_step_port::ExecuteStepPort,
            request_action_execution_port::RequestActionExecutionPort,
            run_shell_step_port::RunShellStepPort,
        },
    },
    domain::{errors::StepError, expression::StepInterpolator},
};

/// Service that executes one step: shell scripts through the shell runner,
/// action references by asking the rest of the system to run them.
pub struct ExecuteStepService {
    action_requester: Box<dyn RequestActionExecutionPort>,
    shell_runner: Box<dyn RunShellStepPort>,
}

impl ExecuteStepService {
    pub fn new(
        action_requester: Box<dyn RequestActionExecutionPort>,
        shell_runner: Box<dyn RunShellStepPort>,
    ) -> Self {
        Self {
            action_requester,
            shell_runner,
        }
    }
}

impl ExecuteStepPort for ExecuteStepService {
    fn execute(&self, request: ExecuteStepRequest<'_>) -> Result<ExecutedStep, StepError> {
        let interpolated = StepInterpolator::interpolate(request.step, request.context)
            .map_err(|error| StepError::new(format!("failed to resolve expressions: {error:?}")))?;

        let response = match interpolated.uses() {
            Some(action_ref) => self.action_requester.execute(ExecuteActionRequest {
                action_ref: action_ref.to_string(),
                step: interpolated.clone(),
                repo_path: request.repo_path.to_path_buf(),
                env: request.env.clone(),
                context: request.context.clone(),
                container: request.container,
            })?,
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
