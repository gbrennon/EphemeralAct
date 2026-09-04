#![allow(dead_code)]
use std::time::Duration;

use ephact::application::{
    dtos::{RunAllWorkflowsRequest, RunSummary},
    ports::inbound::run_all_workflows_port::RunAllWorkflowsPort,
};

pub struct FakeRunAllWorkflowsPort {
    pub result: RunSummary,
}

impl FakeRunAllWorkflowsPort {
    pub fn new(success: bool) -> Self {
        Self {
            result: RunSummary {
                name: "All Workflows".into(),
                job_summaries: vec![],
                success,
                duration: Duration::ZERO,
            },
        }
    }
}

impl RunAllWorkflowsPort for FakeRunAllWorkflowsPort {
    fn execute(
        &self,
        _request: RunAllWorkflowsRequest,
    ) -> Result<RunSummary, Box<dyn std::error::Error>> {
        Ok(self.result.clone())
    }
}
