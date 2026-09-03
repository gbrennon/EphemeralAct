use std::collections::HashMap;

use crate::application::{
    dtos::BuildActionInputEnvironmentRequest,
    ports::outbound::build_action_input_environment_port::BuildActionInputEnvironmentPort,
};

/// Service that exposes an action's inputs the way a real runner does, as
/// `INPUT_<NAME>` variables alongside `GITHUB_ACTION_PATH`.
pub struct BuildActionInputEnvironmentService;

impl BuildActionInputEnvironmentService {
    pub fn new() -> Self {
        Self
    }

    /// Names the environment variable an input is exposed as.
    fn input_variable(name: &str) -> String {
        format!("INPUT_{}", name.to_uppercase().replace(' ', "_"))
    }
}

impl Default for BuildActionInputEnvironmentService {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildActionInputEnvironmentPort for BuildActionInputEnvironmentService {
    fn execute(&self, request: BuildActionInputEnvironmentRequest<'_>) -> HashMap<String, String> {
        let mut action_env = request.env.clone();
        action_env.insert("GITHUB_ACTION_PATH".into(), request.action_path.to_string());
        for (name, value) in request.inputs {
            action_env.insert(Self::input_variable(name), value.clone());
        }
        action_env
    }
}
