use crate::core::{dtos::ExecuteActionResponse, errors::StepError};

/// Result reported back by an event handler.
///
/// Publishing is synchronous: a handler that produces a result the publisher
/// needs — such as the exit status of an executed action — returns it as an
/// outcome, so the publisher never has to reach past the event bus to the
/// handler's port.
#[derive(Debug)]
pub enum EventOutcome {
    /// Reported by the handler of
    /// [`DomainEvent::ActionExecutionRequested`](super::domain_event::DomainEvent::ActionExecutionRequested).
    ActionExecuted(Result<ExecuteActionResponse, StepError>),
}
