use crate::core::{
    ActRunConfig, Repository,
    ports::{inbound::run_act_port::RunActUseCase, outbound::ActExecutor},
    shared_types::ExecutionResult,
};

/// Application service that delegates CI workflow execution to an
/// [`ActExecutor`] adapter.
pub struct RunActService<A: ActExecutor> {
    executor: A,
}

impl<A: ActExecutor> RunActService<A> {
    pub fn new(executor: A) -> Self {
        Self { executor }
    }
}

impl<A: ActExecutor> RunActUseCase for RunActService<A> {
    fn run_act(
        &self,
        config: ActRunConfig,
        repository: Repository,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        self.executor
            .execute_act(&config, &repository)
            .map_err(|e| e.into())
    }
}
