use crate::core::{
    dtos::ContainerCleanupRequest,
    events::{DomainEvent, EventOutcome},
    ports::{
        inbound::{
            container_cleanup_port::ContainerCleanupPort, execute_action_port::ExecuteActionPort,
        },
        outbound::event_publisher::EventPublisherPort,
    },
};

/// In-memory event bus that dispatches domain events to registered handlers.
///
/// Dispatch is synchronous and single-threaded: `publish` routes the event to
/// the handler that owns it and returns whatever that handler reported, which
/// lets a publishing service consume an action's exit status without depending
/// on the action executor's port.
pub struct InMemoryEventBus {
    cleanup_handler: Box<dyn ContainerCleanupPort>,
    action_handler: Box<dyn ExecuteActionPort>,
}

impl InMemoryEventBus {
    pub fn new(
        cleanup_handler: Box<dyn ContainerCleanupPort>,
        action_handler: Box<dyn ExecuteActionPort>,
    ) -> Self {
        Self {
            cleanup_handler,
            action_handler,
        }
    }
}

impl EventPublisherPort for InMemoryEventBus {
    fn publish(&self, event: DomainEvent) -> Vec<EventOutcome> {
        match event {
            DomainEvent::ActRunCompleted(payload) => {
                self.cleanup_handler
                    .execute(ContainerCleanupRequest::new(payload.container_names));
                Vec::new()
            }
            DomainEvent::ActionExecutionRequested(payload) => {
                vec![EventOutcome::ActionExecuted(
                    self.action_handler.execute(payload.request),
                )]
            }
        }
    }
}
