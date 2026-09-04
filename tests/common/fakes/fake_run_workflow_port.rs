#![allow(dead_code)]
use std::time::Duration;

use ephact::application::{
    dtos::{RunSummary, RunWorkflowRequest},
    ports::inbound::run_workflow_port::RunWorkflowPort,
};

pub struct FakeRunWorkflowPort {
    pub result: RunSummary,
}

impl FakeRunWorkflowPort {
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

impl RunWorkflowPort for FakeRunWorkflowPort {
    fn execute(
        &self,
        _request: RunWorkflowRequest,
    ) -> Result<RunSummary, Box<dyn std::error::Error>> {
        Ok(self.result.clone())
    }
}
