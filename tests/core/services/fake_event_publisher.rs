use std::cell::RefCell;

use ephemeral_act::core::{events::DomainEvent, ports::outbound::EventPublisher};

/// Fake event publisher that records published events.
pub(super) struct FakeEventPublisher(RefCell<Vec<DomainEvent>>);

impl FakeEventPublisher {
    pub(super) fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }
}

impl EventPublisher for FakeEventPublisher {
    fn publish(&self, event: DomainEvent) {
        self.0.borrow_mut().push(event);
    }
}
