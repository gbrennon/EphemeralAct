use std::fs::read_to_string;

use crate::{
    application::{
        dtos::LoadActionDefinitionRequest,
        ports::inbound::load_action_definition_port::LoadActionDefinitionPort,
    },
    domain::{errors::StepError, workflow::ActionDefinition},
};

/// Service that reads an action's `action.yml` (or `action.yaml`) and parses it.
pub struct LoadActionDefinitionService;

impl LoadActionDefinitionService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoadActionDefinitionService {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadActionDefinitionPort for LoadActionDefinitionService {
    fn execute(
        &self,
        request: LoadActionDefinitionRequest<'_>,
    ) -> Result<ActionDefinition, StepError> {
        let candidates = [
            request.action_dir.join("action.yml"),
            request.action_dir.join("action.yaml"),
        ];
        let path = candidates
            .iter()
            .find(|candidate| candidate.exists())
            .ok_or_else(|| {
                StepError::new(format!(
                    "action.yml not found in {}",
                    request.action_dir.display()
                ))
            })?;

        let contents = read_to_string(path).map_err(|error| {
            StepError::new(format!("failed to read {}: {error}", path.display()))
        })?;
        serde_yaml::from_str(&contents)
            .map_err(|error| StepError::new(format!("failed to parse {}: {error}", path.display())))
    }
}
