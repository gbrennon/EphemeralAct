use crate::core::events::DomainEvent;

/// Outbound port for publishing domain events.
///
/// Application services depend on this port to notify other parts of the
/// system when significant actions complete.
pub trait EventPublisherPort {
    /// Publish a domain event to all registered subscribers.
    fn publish(&self, event: DomainEvent);
}
