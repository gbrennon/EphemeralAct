#![allow(dead_code)]
use std::{cell::Cell, collections::HashMap, rc::Rc};

use ephact::application::{
    dtos::ReadStepEnvExportsRequest,
    ports::outbound::read_step_env_exports_port::ReadStepEnvExportsPort,
};

/// Returns prepared environment exports, recording that it was consulted.
#[derive(Clone)]
pub struct FakeReadStepEnvExportsPort {
    env: HashMap<String, String>,
    called: Rc<Cell<bool>>,
}

impl FakeReadStepEnvExportsPort {
    pub fn returning(env: HashMap<String, String>) -> Self {
        Self {
            env,
            called: Rc::new(Cell::new(false)),
        }
    }

    pub fn was_called(&self) -> bool {
        self.called.get()
    }
}

impl ReadStepEnvExportsPort for FakeReadStepEnvExportsPort {
    fn execute(&self, _request: ReadStepEnvExportsRequest<'_>) -> HashMap<String, String> {
        self.called.set(true);
        self.env.clone()
    }
}
