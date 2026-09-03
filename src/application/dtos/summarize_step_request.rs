use std::time::Duration;

use crate::{
    application::dtos::ExecutedStep,
    domain::{errors::StepError, workflow::Step},
};

/// Request DTO for the
/// [`SummarizeStepPort`](crate::application::ports::outbound::summarize_step_port::SummarizeStepPort)
/// inbound port.
pub struct SummarizeStepRequest<'a> {
    /// The step as declared, before its expressions were resolved.
    pub step: &'a Step,
    /// What executing the step produced.
    pub outcome: Result<ExecutedStep, StepError>,
    /// How long the step took.
    pub duration: Duration,
}
