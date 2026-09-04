#![allow(dead_code)]
use std::error::Error;

use ephact::application::{
    dtos::{RunAllWorkflowsRequest, RunSummary},
    ports::inbound::run_all_workflows_port::RunAllWorkflowsPort,
};

pub struct StubRunAllWorkflowsPort {
    pub result: Result<RunSummary, String>,
}

impl RunAllWorkflowsPort for StubRunAllWorkflowsPort {
    fn execute(&self, _request: RunAllWorkflowsRequest) -> Result<RunSummary, Box<dyn Error>> {
        self.result.clone().map_err(Box::<dyn Error>::from)
    }
}
