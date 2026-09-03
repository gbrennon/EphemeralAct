use crate::{application::dtos::ExecuteActionResponse, domain::workflow::Step};

/// Outcome of executing one step, with the step its expressions resolved to.
#[derive(Debug)]
pub struct ExecutedStep {
    /// The step as executed, with its expressions resolved.
    pub step: Step,
    /// What the execution reported.
    pub response: ExecuteActionResponse,
}
