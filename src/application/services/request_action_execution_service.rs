use std::sync::Arc;

use crate::{
    application::{
        dtos::{ExecuteActionRequest, ExecuteActionResponse},
        ports::{
            outbound::EventPublisherPort,
            outbound::request_action_execution_port::RequestActionExecutionPort,
        },
    },
    domain::{
        errors::StepError,
        events::{ActionExecutionRequestedPayload, DomainEvent, EventOutcome},
    },
};

/// Service that asks the rest of the system to execute an action by publishing
/// an [`DomainEvent::ActionExecutionRequested`] event.
pub struct RequestActionExecutionService {
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl RequestActionExecutionService {
    pub fn new(event_publisher: Arc<dyn EventPublisherPort>) -> Self {
        Self { event_publisher }
    }
}

impl RequestActionExecutionPort for RequestActionExecutionService {
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError> {
        let action_ref = request.action_ref.clone();
        let outcomes = self
            .event_publisher
            .publish(DomainEvent::ActionExecutionRequested(Box::new(
                ActionExecutionRequestedPayload { request },
            )));

        outcomes
            .into_iter()
            .map(|outcome| match outcome {
                EventOutcome::ActionExecuted(result) => result,
            })
            .next()
            .unwrap_or_else(|| {
                Err(StepError::new(format!(
                    "no handler executed the action '{action_ref}'"
                )))
            })
    }
}
