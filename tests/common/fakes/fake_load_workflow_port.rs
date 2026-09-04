#![allow(dead_code)]
use ephact::application::ports::outbound::load_workflow_port::LoadWorkflowPort;
use parking_lot::Mutex;
use std::sync::Arc;

use ephact::{application::dtos::LoadWorkflowRequest, domain::workflow::Workflow};

/// Parses a prepared YAML document instead of reading one from disk.
#[derive(Clone)]
pub struct FakeLoadWorkflowPort {
    yaml: Result<String, String>,
    loaded_contents: Arc<Mutex<Vec<String>>>,
}

impl FakeLoadWorkflowPort {
    pub fn holding(yaml: &str) -> Self {
        Self {
            yaml: Ok(yaml.to_string()),
            loaded_contents: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            yaml: Err(message.to_string()),
            loaded_contents: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn loaded_contents(&self) -> Vec<String> {
        self.loaded_contents.lock().clone()
    }
}

impl LoadWorkflowPort for FakeLoadWorkflowPort {
    fn execute(
        &self,
        request: LoadWorkflowRequest<'_>,
    ) -> Result<Workflow, Box<dyn std::error::Error>> {
        self.loaded_contents
            .lock()
            .push(request.workflow_content.to_string());
        match &self.yaml {
            Ok(yaml) => Ok(serde_yaml::from_str(yaml)?),
            Err(message) => Err(message.clone().into()),
        }
    }
}
