use crate::application::{
    dtos::{BuildActionInputEnvironmentRequest, BuildActionInputEnvironmentResponse},
    ports::outbound::build_action_input_environment_port::BuildActionInputEnvironmentPort,
};

/// Infrastructure adapter that prepares an action's execution environment
/// following the GitHub Actions specification: `GITHUB_ACTION_PATH` and
/// `INPUT_<NAME>` variables.
pub struct GitHubActionInputEnvironmentAdapter;

impl GitHubActionInputEnvironmentAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Names the environment variable an input is exposed as.
    fn input_variable(name: &str) -> String {
        format!("INPUT_{}", name.to_uppercase().replace(' ', "_"))
    }
}

impl Default for GitHubActionInputEnvironmentAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildActionInputEnvironmentPort for GitHubActionInputEnvironmentAdapter {
    fn execute(
        &self,
        request: BuildActionInputEnvironmentRequest<'_>,
    ) -> BuildActionInputEnvironmentResponse {
        let mut action_env = request.env.clone();
        action_env.insert("GITHUB_ACTION_PATH".into(), request.action_path.to_string());
        for (name, value) in request.inputs {
            action_env.insert(Self::input_variable(name), value.clone());
        }
        BuildActionInputEnvironmentResponse { env: action_env }
    }
}
