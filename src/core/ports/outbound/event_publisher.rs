use crate::core::events::{DomainEvent, EventOutcome};

/// Outbound port for publishing domain events.
///
/// Application services depend on this port to hand work and notifications to
/// the rest of the system without knowing which handlers exist.
///
/// Dispatch is synchronous: `publish` returns once every subscribed handler has
/// run, and each handler that produces a result the publisher needs contributes
/// an [`EventOutcome`]. Events with no interested handler yield no outcomes.
pub trait EventPublisherPort {
    /// Publish a domain event to all registered subscribers and return the
    /// outcomes they reported, in handler order.
    fn publish(&self, event: DomainEvent) -> Vec<EventOutcome>;
}
