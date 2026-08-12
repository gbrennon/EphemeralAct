use std::error::Error;

use ephemeral_act::core::{
    dtos::{RunActRequest, RunSummary},
    ports::inbound::run_act_port::RunActPort,
};

#[allow(dead_code)]
pub struct StubUseCase {
    pub result: Result<RunSummary, String>,
}

impl RunActPort for StubUseCase {
    fn execute(&self, _request: RunActRequest) -> Result<RunSummary, Box<dyn Error>> {
        self.result.clone().map_err(Box::<dyn Error>::from)
    }
}
