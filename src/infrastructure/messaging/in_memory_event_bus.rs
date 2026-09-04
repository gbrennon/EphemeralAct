use crate::{
    application::ports::outbound::EventBusPort, domain::events::DomainEvent,
    infrastructure::containers::ContainerCleanupHandler,
};

/// Event bus that dispatches published domain events to the in-process
/// handlers interested in them.
pub struct InMemoryEventBus {
    cleanup_handler: Box<ContainerCleanupHandler>,
}

impl InMemoryEventBus {
    pub fn new(cleanup_handler: Box<ContainerCleanupHandler>) -> Self {
        Self { cleanup_handler }
    }
}

impl EventBusPort for InMemoryEventBus {
    fn publish(&self, event: DomainEvent) {
        self.cleanup_handler.handle(&event);
    }
}
