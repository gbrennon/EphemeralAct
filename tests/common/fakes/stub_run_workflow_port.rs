#![allow(dead_code)]
use std::error::Error;

use ephact::application::{
    dtos::{RunSummary, RunWorkflowRequest},
    ports::inbound::run_workflow_port::RunWorkflowPort,
};

pub struct StubRunWorkflowPort {
    pub result: Result<RunSummary, String>,
}

impl RunWorkflowPort for StubRunWorkflowPort {
    fn execute(&self, _request: RunWorkflowRequest) -> Result<RunSummary, Box<dyn Error>> {
        self.result.clone().map_err(Box::<dyn Error>::from)
    }
}
