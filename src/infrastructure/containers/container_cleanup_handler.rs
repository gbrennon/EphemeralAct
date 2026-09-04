use std::sync::Arc;

use crate::{
    domain::events::{ActRunCompletedPayload, DomainEvent},
    infrastructure::containers::ContainerRuntimePort,
};

/// Infrastructure handler that cleans up containers when execution finishes.
pub struct ContainerCleanupHandler {
    runtime: Arc<dyn ContainerRuntimePort>,
}

impl ContainerCleanupHandler {
    pub fn new(runtime: Arc<dyn ContainerRuntimePort>) -> Self {
        Self { runtime }
    }

    /// Stops and removes every container a completed run left behind.
    pub fn handle(&self, event: &DomainEvent) {
        let DomainEvent::ActRunCompleted(ActRunCompletedPayload {
            container_names, ..
        }) = event;
        for name in container_names {
            let _ = self.runtime.stop_container(name);
            let _ = self.runtime.remove_container(name);
        }
    }
}
