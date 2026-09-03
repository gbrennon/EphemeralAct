#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ephact::application::{
    dtos::{ReadStepExportsRequest, StepExports},
    ports::outbound::read_step_exports_port::ReadStepExportsPort,
};

/// Hands out the next queued set of exports, or nothing once drained.
#[derive(Clone, Default)]
pub struct FakeReadStepExportsPort {
    queued: Rc<RefCell<Vec<(Vec<String>, HashMap<String, String>)>>>,
    calls: Rc<RefCell<usize>>,
}

impl FakeReadStepExportsPort {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queueing(exports: Vec<(Vec<String>, HashMap<String, String>)>) -> Self {
        Self {
            queued: Rc::new(RefCell::new(exports)),
            calls: Rc::new(RefCell::new(0)),
        }
    }

    pub fn calls(&self) -> usize {
        *self.calls.borrow()
    }
}

impl ReadStepExportsPort for FakeReadStepExportsPort {
    fn execute(&self, _request: ReadStepExportsRequest<'_>) -> StepExports {
        *self.calls.borrow_mut() += 1;
        let mut queued = self.queued.borrow_mut();
        if queued.is_empty() {
            return StepExports {
                path_additions: Vec::new(),
                env: HashMap::new(),
            };
        }
        let (path_additions, env) = queued.remove(0);
        StepExports {
            path_additions,
            env,
        }
    }
}
