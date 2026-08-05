use crate::core::{ActRunConfig, Repository, shared_types::ExecutionResult};

pub trait RunActUseCase {
    fn run_act(
        &self,
        config: ActRunConfig,
        repository: Repository,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>>;
}
