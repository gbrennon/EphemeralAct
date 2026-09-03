use crate::{
    application::{
        dtos::{ExecuteActionRequest, RunCompositeStepRequest, RunShellStepRequest},
        ports::{
            inbound::{
                run_composite_step_port::RunCompositeStepPort,
                run_shell_step_port::RunShellStepPort,
            },
            outbound::ExecResult,
        },
    },
    domain::errors::StepError,
};

/// Service that runs one step of a composite action, either as a script in the
/// job's container or as a nested action through the executor it was handed.
pub struct RunCompositeStepService {
    shell_runner: Box<dyn RunShellStepPort>,
}

impl RunCompositeStepService {
    pub fn new(shell_runner: Box<dyn RunShellStepPort>) -> Self {
        Self { shell_runner }
    }
}

impl RunCompositeStepPort for RunCompositeStepService {
    fn execute(&self, request: RunCompositeStepRequest<'_>) -> Result<ExecResult, StepError> {
        let action_request = request.action_request;

        match request.step.uses() {
            Some(nested) => request
                .nested_executor
                .execute(ExecuteActionRequest {
                    action_ref: nested.to_string(),
                    step: request.step.clone(),
                    repo_path: action_request.repo_path.clone(),
                    env: action_request.env.clone(),
                    context: request.context.clone(),
                    container: action_request.container.clone(),
                })
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
