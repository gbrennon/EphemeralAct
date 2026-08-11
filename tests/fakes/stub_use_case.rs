use std::error::Error;

use ephemeral_act::core::{
    ActRunConfig, Repository, ports::inbound::run_act_port::RunActUseCase,
    shared_types::ExecutionResult,
};

#[allow(dead_code)]
pub struct StubUseCase {
    pub result: Result<ExecutionResult, String>,
}

impl RunActUseCase for StubUseCase {
    fn run_act(
        &self,
        _config: ActRunConfig,
        _repository: Repository,
    ) -> Result<ExecutionResult, Box<dyn Error>> {
        self.result.clone().map_err(Box::<dyn Error>::from)
    }
}
