use crate::{application::dtos::BuildRunContextRequest, domain::expression::EvalContext};

/// Inbound port for building the expression context a run is evaluated against.
pub trait BuildRunContextPort {
    /// Builds the run's `secrets`, `inputs`, `github`, and `runner` contexts.
    fn execute(&self, request: BuildRunContextRequest<'_>) -> EvalContext;
}
