#![allow(dead_code)]
use ephact::infrastructure::steps::read_step_path_exports_port::ReadStepPathExportsPort;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use ephact::application::dtos::ReadStepPathExportsRequest;

/// Returns prepared path additions, recording that it was consulted.
#[derive(Clone)]
pub struct FakeReadStepPathExportsPort {
    additions: Vec<String>,
    called: Arc<AtomicBool>,
}

impl FakeReadStepPathExportsPort {
    pub fn returning(additions: Vec<String>) -> Self {
        Self {
            additions,
            called: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn was_called(&self) -> bool {
        self.called.load(Ordering::SeqCst)
    }
}

impl ReadStepPathExportsPort for FakeReadStepPathExportsPort {
    fn execute(&self, _request: ReadStepPathExportsRequest<'_>) -> Vec<String> {
        self.called.store(true, Ordering::SeqCst);
        self.additions.clone()
    }
}
