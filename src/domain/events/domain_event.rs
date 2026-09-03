/// Domain events published by application services.
///
/// These events are dispatched in-memory via the [`EventPublisherPort`] outbound
/// port. Infrastructure handlers subscribe to specific event variants to
/// perform side effects (e.g. container cleanup) or to run work the publishing
/// service must not depend on directly (e.g. action execution).
///
/// [`EventPublisherPort`]: crate::application::ports::outbound::event_publisher::EventPublisherPort
#[derive(Debug, Clone)]
pub enum DomainEvent {
    /// Published when a workflow run completes (success or failure).
    ActRunCompleted(super::act_run_completed_payload::ActRunCompletedPayload),

    /// Published when a step references an action that has to be resolved and
    /// executed. The payload is boxed because it carries a whole execution
    /// request, which dwarfs the other variants.
    ActionExecutionRequested(
        Box<super::action_execution_requested_payload::ActionExecutionRequestedPayload>,
    ),
}
