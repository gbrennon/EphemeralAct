use crate::core::events::DomainEvent;

/// Outbound port for publishing domain events.
///
/// Application services depend on this port to notify other parts of the
/// system when significant actions complete. Infrastructure adapters
/// (e.g. an in-memory event bus) implement this trait and route events
/// to registered handlers.
pub trait EventPublisher {
    /// Publish a domain event to all registered subscribers.
    fn publish(&self, event: DomainEvent);
}
