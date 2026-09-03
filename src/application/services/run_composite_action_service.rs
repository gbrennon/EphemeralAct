use std::collections::HashMap;

use serde_json::Value;

use crate::{
    application::{
        dtos::{ExecuteActionResponse, RunCompositeActionRequest, RunCompositeStepRequest},
        ports::inbound::{
            run_composite_action_port::RunCompositeActionPort,
            run_composite_step_port::RunCompositeStepPort,
        },
    },
    domain::{
        errors::StepError,
        expression::{EvalContext, StepInterpolator},
    },
};

/// Service that runs a composite action's steps in order, accumulating their
/// output and stopping at the first one that fails.
pub struct RunCompositeActionService {
    step_runner: Box<dyn RunCompositeStepPort>,
}

impl RunCompositeActionService {
    pub fn new(step_runner: Box<dyn RunCompositeStepPort>) -> Self {
        Self { step_runner }
    }

    /// Returns a copy of `context` whose `inputs` are the action's own.
    fn context_with_inputs(context: &EvalContext, inputs: &HashMap<String, String>) -> EvalContext {
        let mut action_context = context.clone();
        action_context.inputs = Value::Object(
            inputs
                .iter()
                .map(|(name, value)| (name.clone(), Value::String(value.clone())))
                .collect(),
        );
        action_context
    }
}

impl RunCompositeActionPort for RunCompositeActionService {
    fn execute(
        &self,
        request: RunCompositeActionRequest<'_>,
    ) -> Result<ExecuteActionResponse, StepError> {
        let context = Self::context_with_inputs(&request.action_request.context, request.inputs);
        let mut stdout = String::new();
        let mut stderr = String::new();

        for step in request.steps {
            let interpolated =
                StepInterpolator::interpolate(step, &context).map_err(|error| StepError {
                    message: format!("failed to resolve expressions: {error:?}"),
                    stdout: stdout.clone(),
                    stderr: stderr.clone(),
                })?;

            let outcome = self.step_runner.execute(RunCompositeStepRequest {
                step: &interpolated,
                action_dir: request.action_dir,
                action_request: request.action_request,
                context: &context,
                nested_executor: request.nested_executor,
            });

            match outcome {
                Ok(result) => {
                    stdout.push_str(&result.stdout);
                    stderr.push_str(&result.stderr);
                    if result.exit_code != 0 {
                        return Ok(ExecuteActionResponse {
                            exit_code: result.exit_code,
                            stdout,
                            stderr,
                        });
                    }
                }
                Err(error) => {
                    return Err(StepError {
                        message: error.message,
                        stdout: format!("{stdout}{}", error.stdout),
                        stderr: format!("{stderr}{}", error.stderr),
                    });
                }
            }
        }

        Ok(ExecuteActionResponse {
            exit_code: 0,
            stdout,
            stderr,
        })
    }
}
