use std::sync::Arc;

use crate::{
    core::{
        ports::inbound::run_act_port::RunActUseCase,
        services::{
            container_cleanup_service::ContainerCleanupService, run_act_service::RunActService,
        },
    },
    infrastructure::{
        PlatformImageMapper, in_memory_event_bus::InMemoryEventBus,
        runners::ContainerRuntimeAdapter,
    },
};

/// Dependency-injection container that constructs and wires all application
/// dependencies. Returns a fully-wired [`RunActUseCase`] ready for the
/// presentation layer to consume.
pub struct Container;

impl Container {
    /// Builds the application service graph and returns the entry-point
    /// use case.
    pub fn build() -> impl RunActUseCase {
        let runtime = Arc::new(
            ContainerRuntimeAdapter::detect()
                .expect("no container runtime available (Docker or Podman required)"),
        );
        let image_mapper = PlatformImageMapper;
        let cleanup_service = ContainerCleanupService::new(runtime.clone());
        let event_bus = InMemoryEventBus::new(Box::new(cleanup_service));
        RunActService::new(runtime, image_mapper, event_bus)
    }
}
