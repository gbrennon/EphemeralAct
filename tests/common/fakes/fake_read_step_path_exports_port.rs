#![allow(dead_code)]
use std::{cell::Cell, rc::Rc};

use ephact::application::{
    dtos::ReadStepPathExportsRequest,
    ports::inbound::read_step_path_exports_port::ReadStepPathExportsPort,
};

/// Returns prepared path additions, recording that it was consulted.
#[derive(Clone)]
pub struct FakeReadStepPathExportsPort {
    additions: Vec<String>,
    called: Rc<Cell<bool>>,
}

impl FakeReadStepPathExportsPort {
    pub fn returning(additions: Vec<String>) -> Self {
        Self {
            additions,
            called: Rc::new(Cell::new(false)),
        }
    }

    pub fn was_called(&self) -> bool {
        self.called.get()
    }
}

impl ReadStepPathExportsPort for FakeReadStepPathExportsPort {
    fn execute(&self, _request: ReadStepPathExportsRequest<'_>) -> Vec<String> {
        self.called.set(true);
        self.additions.clone()
    }
}
