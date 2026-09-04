use crate::application::ports::outbound::build_step_context_port::BuildStepContextPort;
use serde_json::Value;

use crate::{application::dtos::BuildStepContextRequest, domain::expression::EvalContext};

/// Service that mirrors a step's environment into the `env` expression context.
pub struct BuildStepContextService;

impl BuildStepContextService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BuildStepContextService {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildStepContextPort for BuildStepContextService {
    fn execute(&self, request: BuildStepContextRequest<'_>) -> EvalContext {
        let mut step_context = request.context.clone();
        step_context.env = Value::Object(
            request
                .env
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect(),
        );
        step_context
    }
}
