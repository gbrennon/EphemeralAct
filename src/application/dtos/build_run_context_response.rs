use crate::domain::expression::EvalContext;

/// Response DTO for the
/// [`BuildRunContextPort`](crate::application::ports::inbound::build_run_context_port::BuildRunContextPort)
/// outbound port.
#[derive(Debug, Clone)]
pub struct BuildRunContextResponse {
    /// The evaluated run context.
    pub context: EvalContext,
}
