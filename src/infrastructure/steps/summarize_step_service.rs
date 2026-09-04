use crate::application::ports::outbound::summarize_step_port::SummarizeStepPort;
use crate::{
    application::dtos::{StepSummary, SummarizeStepRequest, SummarizedStep},
    domain::workflow::Step,
};

/// Service that turns a step's outcome into its run-summary entry, deciding
/// whether the outcome fails the job.
pub struct SummarizeStepService;

impl SummarizeStepService {
    pub fn new() -> Self {
        Self
    }

    /// Names a step the way the run summary reports it.
    fn step_label(step: &Step) -> String {
        step.name
            .as_deref()
            .or(step.id.as_deref())
            .or(step.run.as_deref())
            .or(step.uses.as_deref())
            .unwrap_or("unnamed step")
            .to_string()
    }
}

impl Default for SummarizeStepService {
    fn default() -> Self {
        Self::new()
    }
}

impl SummarizeStepPort for SummarizeStepService {
    fn execute(&self, request: SummarizeStepRequest<'_>) -> SummarizedStep {
        let step_type = request.step.step_type();
        let continue_on_error = request.step.continues_on_error();

        let (exit_code, stdout, stderr, name, fails_job) = match request.outcome {
            Ok(executed) => {
                let fails_job = executed.response.exit_code != 0 && !continue_on_error;
                (
                    Some(executed.response.exit_code),
                    executed.response.stdout,
                    executed.response.stderr,
                    Self::step_label(&executed.step),
                    fails_job,
                )
            }
            Err(error) => (
                None,
                error.stdout,
                format!("step error: {}\n{}", error.message, error.stderr),
                Self::step_label(request.step),
                !continue_on_error,
            ),
        };

        SummarizedStep {
            summary: StepSummary {
                name,
                step_type,
                exit_code,
                continue_on_error,
                duration: request.duration,
                stdout,
                stderr,
            },
            fails_job,
        }
    }
}
