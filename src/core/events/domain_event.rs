/// Domain events published by application services.
///
/// These events are dispatched in-memory via the [`EventPublisher`] outbound
/// port. Infrastructure handlers subscribe to specific event variants to
/// perform side effects (e.g. container cleanup).
///
/// [`EventPublisher`]: crate::core::ports::outbound::event_publisher::EventPublisher
#[derive(Debug, Clone)]
pub enum DomainEvent {
    /// Published when a workflow run completes (success or failure).
    ActRunCompleted(super::act_run_completed_payload::ActRunCompletedPayload),
}
