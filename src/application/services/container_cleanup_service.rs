use crate::application::{
    dtos::ContainerCleanupRequest,
    ports::{
        inbound::container_cleanup_port::ContainerCleanupPort,
        outbound::container_runtime::ContainerRuntimePort,
    },
};

/// Application service that reacts to workflow completion by cleaning up
/// containers created during the run.
///
/// Implements [`ContainerCleanupPort`] - stops and removes containers
/// but does NOT delete cached images.
pub struct ContainerCleanupService<R: ContainerRuntimePort> {
    runtime: R,
}

impl<R: ContainerRuntimePort> ContainerCleanupService<R> {
    pub fn new(runtime: R) -> Self {
        Self { runtime }
    }
}

impl<R: ContainerRuntimePort> ContainerCleanupPort for ContainerCleanupService<R> {
    fn execute(&self, request: ContainerCleanupRequest) {
        for name in &request.container_names {
            let _ = self.runtime.stop_container(name);
            let _ = self.runtime.remove_container(name);
        }
    }
}
