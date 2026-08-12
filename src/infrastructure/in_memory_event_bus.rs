use crate::core::{
    dtos::ContainerCleanupRequest,
    events::DomainEvent,
    ports::{
        inbound::container_cleanup_port::ContainerCleanupPort,
        outbound::event_publisher::EventPublisherPort,
    },
};

/// In-memory event bus that dispatches domain events to registered handlers.
///
/// Currently routes [`DomainEvent::ActRunCompleted`] directly to a
/// [`ContainerCleanupPort`] handler. Additional handlers can be added
/// by extending the `publish` match arm.
pub struct InMemoryEventBus {
    cleanup_handler: Box<dyn ContainerCleanupPort>,
}

impl InMemoryEventBus {
    pub fn new(cleanup_handler: Box<dyn ContainerCleanupPort>) -> Self {
        Self { cleanup_handler }
    }
}

impl EventPublisherPort for InMemoryEventBus {
    fn publish(&self, event: DomainEvent) {
        match event {
            DomainEvent::ActRunCompleted(payload) => {
                self.cleanup_handler
                    .execute(ContainerCleanupRequest::new(payload.container_names));
            }
        }
    }
}
