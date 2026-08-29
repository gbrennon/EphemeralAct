#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephemeral_act::core::{events::DomainEvent, ports::outbound::EventPublisherPort};

#[derive(Clone)]
pub struct FakeEventPublisher(Rc<RefCell<Vec<DomainEvent>>>);

impl FakeEventPublisher {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }

    pub fn events(&self) -> Vec<DomainEvent> {
        self.0.borrow().clone()
    }
}

impl EventPublisherPort for FakeEventPublisher {
    fn publish(&self, event: DomainEvent) {
        self.0.borrow_mut().push(event);
    }
}
