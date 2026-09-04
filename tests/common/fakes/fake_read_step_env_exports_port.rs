#![allow(dead_code)]
use ephact::infrastructure::steps::read_step_env_exports_port::ReadStepEnvExportsPort;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use ephact::application::dtos::ReadStepEnvExportsRequest;

/// Returns prepared environment exports, recording that it was consulted.
#[derive(Clone)]
pub struct FakeReadStepEnvExportsPort {
    env: HashMap<String, String>,
    called: Arc<AtomicBool>,
}

impl FakeReadStepEnvExportsPort {
    pub fn returning(env: HashMap<String, String>) -> Self {
        Self {
            env,
            called: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn was_called(&self) -> bool {
        self.called.load(Ordering::SeqCst)
    }
}

impl ReadStepEnvExportsPort for FakeReadStepEnvExportsPort {
    fn execute(&self, _request: ReadStepEnvExportsRequest<'_>) -> HashMap<String, String> {
        self.called.store(true, Ordering::SeqCst);
        self.env.clone()
    }
}
