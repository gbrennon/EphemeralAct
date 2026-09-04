use crate::application::dtos::{BuildRunContextRequest, BuildRunContextResponse};

/// Inbound port for building the expression context a run is evaluated against.
pub trait BuildRunContextPort: Send + Sync {
    /// Builds the run's `secrets`, `inputs`, `github`, and `runner` contexts.
    fn execute(&self, request: BuildRunContextRequest<'_>) -> BuildRunContextResponse;
}
