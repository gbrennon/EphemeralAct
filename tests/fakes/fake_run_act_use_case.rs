use ephemeral_act::core::{
    ActRunConfig, Repository, ports::inbound::run_act_port::RunActUseCase,
    shared_types::ExecutionResult,
};

#[allow(dead_code)]
pub struct FakeRunActUseCase {
    pub result: ExecutionResult,
}

#[allow(dead_code)]
impl FakeRunActUseCase {
    pub fn new(success: bool) -> Self {
        Self {
            result: ExecutionResult {
                success,
                stdout: String::new(),
                stderr: String::new(),
            },
        }
    }
}

impl RunActUseCase for FakeRunActUseCase {
    fn run_act(
        &self,
        _config: ActRunConfig,
        _repository: Repository,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        Ok(self.result.clone())
    }
}
