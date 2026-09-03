#![allow(dead_code)]
use std::time::Duration;

use ephact::application::{
    dtos::{RunActRequest, RunSummary},
    ports::inbound::run_act_port::RunActPort,
};

pub struct FakeRunActPort {
    pub result: RunSummary,
}

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
