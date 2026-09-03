use std::{error::Error, fs::read_to_string};

use crate::{
    application::{
        dtos::LoadWorkflowRequest, ports::inbound::load_workflow_port::LoadWorkflowPort,
    },
    domain::workflow::Workflow,
};

/// Service that reads a workflow file from disk and parses it.
pub struct LoadWorkflowService;

impl LoadWorkflowService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoadWorkflowService {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadWorkflowPort for LoadWorkflowService {
    fn execute(&self, request: LoadWorkflowRequest<'_>) -> Result<Workflow, Box<dyn Error>> {
        let yaml = read_to_string(request.workflow_file)?;
        Ok(serde_yaml::from_str(&yaml)?)
    }
}
