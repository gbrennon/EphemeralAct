use crate::application::dtos::ExecuteActionRequest;

/// Payload for [`DomainEvent::ActionExecutionRequested`].
///
/// Published by the workflow runner when a step references an action instead of
/// a shell script. The handler subscribed to this event owns action resolution
/// and execution, and reports back through
/// [`EventOutcome::ActionExecuted`](super::event_outcome::EventOutcome::ActionExecuted).
///
/// [`DomainEvent::ActionExecutionRequested`]: super::domain_event::DomainEvent::ActionExecutionRequested
#[derive(Debug, Clone)]
pub struct ActionExecutionRequestedPayload {
    /// Everything the handler needs to run the action.
    pub request: ExecuteActionRequest,
}
