use std::cell::RefCell;

use ephemeral_act::core::{events::DomainEvent, ports::outbound::EventPublisher};

#[allow(dead_code)]
pub struct FakeEventPublisher(RefCell<Vec<DomainEvent>>);

#[allow(dead_code)]
impl FakeEventPublisher {
    pub fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }
}

impl EventPublisher for FakeEventPublisher {
    fn publish(&self, event: DomainEvent) {
        self.0.borrow_mut().push(event);
    }
}
