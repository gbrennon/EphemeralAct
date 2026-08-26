use std::time::Duration;

use ephemeral_act::core::{
    dtos::{RunActRequest, RunSummary},
    ports::inbound::run_act_port::RunActPort,
};

#[allow(dead_code)]
pub struct FakeRunActPort {
    pub result: RunSummary,
}

#[allow(dead_code)]
impl FakeRunActPort {
    pub fn new(success: bool) -> Self {
        Self {
            result: RunSummary {
                name: "test".into(),
                job_summaries: vec![],
                success,
                duration: Duration::ZERO,
            },
        }
    }
}

impl RunActPort for FakeRunActPort {
    fn execute(&self, _request: RunActRequest) -> Result<RunSummary, Box<dyn std::error::Error>> {
        Ok(self.result.clone())
    }
}
