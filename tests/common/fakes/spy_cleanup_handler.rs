#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephact::core::{
    dtos::ContainerCleanupRequest, ports::inbound::container_cleanup_port::ContainerCleanupPort,
};

/// Cleanup handler that records the container names of every request it
/// receives. Every clone observes the same recording.
#[derive(Clone, Default)]
pub struct SpyCleanupHandler {
    cleaned_up: Rc<RefCell<Vec<Vec<String>>>>,
}

impl SpyCleanupHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cleaned_up(&self) -> Vec<Vec<String>> {
        self.cleaned_up.borrow().clone()
    }
}

impl ContainerCleanupPort for SpyCleanupHandler {
    fn execute(&self, request: ContainerCleanupRequest) {
        self.cleaned_up.borrow_mut().push(request.container_names);
    }
}
