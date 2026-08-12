use std::time::Duration;

use ephemeral_act::core::{
    dtos::{RunActRequest, RunSummary},
    ports::inbound::run_act_port::RunActPort,
};

#[allow(dead_code)]
pub struct FakeRunActUseCase {
    pub result: RunSummary,
}

#[allow(dead_code)]
impl FakeRunActUseCase {
    pub fn new(success: bool) -> Self {
        Self {
            result: RunSummary {
                name: None,
                job_summaries: vec![],
                success,
                total_duration: Duration::ZERO,
            },
        }
    }
}

impl RunActPort for FakeRunActUseCase {
    fn execute(&self, _request: RunActRequest) -> Result<RunSummary, Box<dyn std::error::Error>> {
        Ok(self.result.clone())
    }
}
