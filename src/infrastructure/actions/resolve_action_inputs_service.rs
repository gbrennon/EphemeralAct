use crate::application::ports::outbound::resolve_action_inputs_port::ResolveActionInputsPort;
use std::collections::HashMap;

use crate::application::dtos::ResolveActionInputsRequest;

/// Service that resolves the inputs an action runs with, overlaying the step's
/// `with:` values on the defaults the action declared.
pub struct ResolveActionInputsService;

impl ResolveActionInputsService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResolveActionInputsService {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolveActionInputsPort for ResolveActionInputsService {
    fn execute(&self, request: ResolveActionInputsRequest<'_>) -> HashMap<String, String> {
        let mut inputs: HashMap<String, String> = request
            .definition
            .inputs
            .iter()
            .filter_map(|(name, input)| {
                input
                    .default
                    .as_ref()
                    .map(|default| (name.clone(), default.clone()))
            })
            .collect();
        inputs.extend(
            request
                .step
                .with
                .iter()
                .map(|(name, value)| (name.clone(), value.clone())),
        );
        inputs
    }
}
