#![allow(dead_code)]
use ephemeral_act::core::{
    dtos::{ListWorkflowsRequest, ListWorkflowsResponse},
    ports::inbound::list_workflows_port::ListWorkflowsPort,
};

pub struct FakeListWorkflowsPort;

impl FakeListWorkflowsPort {
    pub fn new() -> Self {
        Self
    }
}

impl ListWorkflowsPort for FakeListWorkflowsPort {
    fn execute(
        &self,
        _request: ListWorkflowsRequest,
    ) -> Result<ListWorkflowsResponse, Box<dyn std::error::Error>> {
        Ok(ListWorkflowsResponse::new(vec![]))
    }
}
