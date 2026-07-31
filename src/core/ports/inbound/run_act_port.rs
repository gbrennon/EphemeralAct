use crate::core::shared_types::ExecutionResult;
use crate::core::{ActRunConfig, Repository};

pub trait RunActUseCase {
    fn run_act(
        &self,
        config: ActRunConfig,
        repository: Repository,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>>;
}
