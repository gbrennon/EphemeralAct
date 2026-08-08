use crate::core::ports::{
    inbound::container_cleanup_port::ContainerCleanupUseCase,
    outbound::container_runtime::ContainerRuntime,
};

/// Application service that reacts to workflow completion by cleaning up
/// containers created during the run.
///
/// Implements [`ContainerCleanupUseCase`] — stops containers
/// but does NOT delete cached images.
pub struct ContainerCleanupService<R: ContainerRuntime> {
    runtime: R,
}

impl<R: ContainerRuntime> ContainerCleanupService<R> {
    pub fn new(runtime: R) -> Self {
        Self { runtime }
    }
}

impl<R: ContainerRuntime> ContainerCleanupUseCase for ContainerCleanupService<R> {
    fn handle_act_run_completed(&self, container_names: &[String]) {
        for name in container_names {
            let _ = self.runtime.stop_container(name);
            eprintln!("Container stopped: {name}");
        }
    }
}
