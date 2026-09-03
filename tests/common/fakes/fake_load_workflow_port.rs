#![allow(dead_code)]
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ephact::{
    application::{
        dtos::LoadWorkflowRequest, ports::inbound::load_workflow_port::LoadWorkflowPort,
    },
    domain::workflow::Workflow,
};

/// Parses a prepared YAML document instead of reading one from disk.
#[derive(Clone)]
pub struct FakeLoadWorkflowPort {
    yaml: Result<String, String>,
    loaded: Rc<RefCell<Vec<PathBuf>>>,
}

impl FakeLoadWorkflowPort {
    pub fn holding(yaml: &str) -> Self {
        Self {
            yaml: Ok(yaml.to_string()),
            loaded: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            yaml: Err(message.to_string()),
            loaded: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn loaded(&self) -> Vec<PathBuf> {
        self.loaded.borrow().clone()
    }
}

impl LoadWorkflowPort for FakeLoadWorkflowPort {
    fn execute(
        &self,
        request: LoadWorkflowRequest<'_>,
    ) -> Result<Workflow, Box<dyn std::error::Error>> {
        self.loaded
            .borrow_mut()
            .push(request.workflow_file.to_path_buf());
        match &self.yaml {
            Ok(yaml) => Ok(serde_yaml::from_str(yaml)?),
            Err(message) => Err(message.clone().into()),
        }
    }
}
