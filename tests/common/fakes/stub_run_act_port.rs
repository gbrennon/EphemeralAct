#![allow(dead_code)]
use std::error::Error;

use ephemeral_act::core::{
    dtos::{RunActRequest, RunSummary},
    ports::inbound::run_act_port::RunActPort,
};

pub struct StubRunActPort {
    pub result: Result<RunSummary, String>,
}

impl RunActPort for StubRunActPort {
    fn execute(&self, _request: RunActRequest) -> Result<RunSummary, Box<dyn Error>> {
        self.result.clone().map_err(Box::<dyn Error>::from)
    }
}
