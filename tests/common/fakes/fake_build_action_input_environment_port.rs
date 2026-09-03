#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ephact::application::{
    dtos::{BuildActionInputEnvironmentRequest, BuildActionInputEnvironmentResponse},
    ports::outbound::build_action_input_environment_port::BuildActionInputEnvironmentPort,
};

/// Returns a prepared environment, recording the action paths it was given.
#[derive(Clone)]
pub struct FakeBuildActionInputEnvironmentPort {
    env: HashMap<String, String>,
    action_paths: Rc<RefCell<Vec<String>>>,
}

impl FakeBuildActionInputEnvironmentPort {
    pub fn returning(env: HashMap<String, String>) -> Self {
        Self {
            env,
            action_paths: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn action_paths(&self) -> Vec<String> {
        self.action_paths.borrow().clone()
    }
}

impl BuildActionInputEnvironmentPort for FakeBuildActionInputEnvironmentPort {
    fn execute(
        &self,
        request: BuildActionInputEnvironmentRequest<'_>,
    ) -> BuildActionInputEnvironmentResponse {
        self.action_paths
            .borrow_mut()
            .push(request.action_path.to_string());
        BuildActionInputEnvironmentResponse {
            env: self.env.clone(),
        }
    }
}
