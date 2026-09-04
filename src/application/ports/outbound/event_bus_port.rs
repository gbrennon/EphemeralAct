use crate::domain::events::DomainEvent;

/// Outbound port representing the event bus.
///
/// Publishes domain events (representing facts of something that happened in the past)
/// to be handled by interested event handlers/subscribers in the infrastructure layer.
pub trait EventBusPort: Send + Sync {
    /// Publishes a domain event.
    fn publish(&self, event: DomainEvent);
}
