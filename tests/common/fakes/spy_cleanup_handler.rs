#![allow(dead_code)]
use ephact::infrastructure::containers::container_cleanup_port::ContainerCleanupPort;
use parking_lot::Mutex;
use std::sync::Arc;

use ephact::application::dtos::ContainerCleanupRequest;

/// Cleanup handler that records the container names of every request it
/// receives. Every clone observes the same recording.
#[derive(Clone, Default)]
pub struct SpyCleanupHandler {
    cleaned_up: Arc<Mutex<Vec<Vec<String>>>>,
}

impl SpyCleanupHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cleaned_up(&self) -> Vec<Vec<String>> {
        self.cleaned_up.lock().clone()
    }
}

impl ContainerCleanupPort for SpyCleanupHandler {
    fn execute(&self, request: ContainerCleanupRequest) {
        self.cleaned_up.lock().push(request.container_names);
    }
}
