use crate::core::{
    events::DomainEvent,
    ports::{
        inbound::container_cleanup_port::ContainerCleanupUseCase,
        outbound::event_publisher::EventPublisher,
    },
};

/// In-memory event bus that dispatches domain events to registered handlers.
///
/// Currently routes [`DomainEvent::ActRunCompleted`] directly to a
/// [`ContainerCleanupUseCase`] handler. Additional handlers can be added
/// by extending the `publish` match arm.
pub struct InMemoryEventBus {
    cleanup_handler: Box<dyn ContainerCleanupUseCase>,
}

impl InMemoryEventBus {
    pub fn new(cleanup_handler: Box<dyn ContainerCleanupUseCase>) -> Self {
        Self { cleanup_handler }
    }
}

impl EventPublisher for InMemoryEventBus {
    fn publish(&self, event: DomainEvent) {
        match event {
            DomainEvent::ActRunCompleted(payload) => {
                self.cleanup_handler
                    .handle_act_run_completed(&payload.container_names);
            }
        }
    }
}
