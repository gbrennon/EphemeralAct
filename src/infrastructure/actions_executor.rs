use crate::core::ports::outbound::ActExecutor;
use crate::core::shared_types::ExecutionResult;
use crate::core::{ActRunConfig, Repository};
use crate::infrastructure::act_wrappers::forgejo_act_wrapper::ForgejoActWrapper;
use crate::infrastructure::act_wrappers::github_act_wrapper::GitHubActWrapper;
use crate::infrastructure::act_wrappers::CiPlatform;

/// Dispatches workflow execution to the correct CI adapter using pattern matching
/// on the detected platform directory layout.
pub struct ActionsExecutor {
    github: GitHubActWrapper,
    forgejo: ForgejoActWrapper,
}

impl ActionsExecutor {
    pub fn new() -> Self {
        Self {
            github: GitHubActWrapper,
            forgejo: ForgejoActWrapper,
        }
    }
}

impl Default for ActionsExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ActExecutor for ActionsExecutor {
    fn execute_act(
        &self,
        config: &ActRunConfig,
        repository: &Repository,
    ) -> Result<ExecutionResult, String> {
        let platform = CiPlatform::detect(repository);
        match platform {
            CiPlatform::Forgejo => self.forgejo.execute_act(config, repository),
            CiPlatform::GitHub => self.github.execute_act(config, repository),
        }
    }
}
