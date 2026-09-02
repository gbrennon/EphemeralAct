#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephemeral_act::core::{
    events::{DomainEvent, EventOutcome},
    ports::{inbound::ExecuteActionPort, outbound::EventPublisherPort},
};

/// Records every published event and, when an action handler is registered,
/// dispatches action execution requests to it the way the real bus does.
#[derive(Clone)]
pub struct FakeEventPublisher {
    events: Rc<RefCell<Vec<DomainEvent>>>,
    action_handler: Option<Rc<dyn ExecuteActionPort>>,
}

impl FakeEventPublisher {
    /// Creates a publisher with no subscribed handlers.
    pub fn new() -> Self {
        Self {
            events: Rc::new(RefCell::new(Vec::new())),
            action_handler: None,
        }
    }

    /// Creates a publisher that routes action execution requests to `handler`.
    pub fn with_action_handler(handler: Rc<dyn ExecuteActionPort>) -> Self {
        Self {
            events: Rc::new(RefCell::new(Vec::new())),
            action_handler: Some(handler),
        }
    }

    pub fn events(&self) -> Vec<DomainEvent> {
        self.events.borrow().clone()
    }
}

impl EventPublisherPort for FakeEventPublisher {
    fn publish(&self, event: DomainEvent) -> Vec<EventOutcome> {
        self.events.borrow_mut().push(event.clone());

        match (event, self.action_handler.as_ref()) {
            (DomainEvent::ActionExecutionRequested(payload), Some(handler)) => {
                vec![EventOutcome::ActionExecuted(
                    handler.execute(payload.request),
                )]
            }
            _ => Vec::new(),
        }
    }
}
