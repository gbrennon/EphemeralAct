use std::cell::RefCell;
use std::rc::Rc;

use ephemeral_act::core::{
    events::{ActRunCompletedPayload, DomainEvent},
    ports::{
        inbound::container_cleanup_port::ContainerCleanupUseCase,
        outbound::event_publisher::EventPublisher,
    },
};
use ephemeral_act::infrastructure::InMemoryEventBus;

struct SpyCleanupHandler {
    called_with: RefCell<Vec<Vec<String>>>,
}

impl SpyCleanupHandler {
    fn new() -> Self {
        Self {
            called_with: RefCell::new(Vec::new()),
        }
    }
}

impl ContainerCleanupUseCase for SpyCleanupHandler {
    fn handle_act_run_completed(&self, container_names: &[String]) {
        self.called_with
            .borrow_mut()
            .push(container_names.to_vec());
    }
}

#[test]
fn new_creates_event_bus() {
    let handler = Rc::new(SpyCleanupHandler::new());
    let _bus = InMemoryEventBus::new(Box::new(SpyHandler(handler.clone())));
}

#[test]
fn publish_act_run_completed_dispatches_to_handler() {
    let handler = Rc::new(SpyCleanupHandler::new());
    let bus = InMemoryEventBus::new(Box::new(SpyHandler(handler.clone())));
    let names = vec!["container-a".into(), "container-b".into()];

    bus.publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
        container_names: names.clone(),
        success: true,
    }));

    let calls = handler.called_with.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], names);
}

#[test]
fn publish_act_run_completed_with_empty_names() {
    let handler = Rc::new(SpyCleanupHandler::new());
    let bus = InMemoryEventBus::new(Box::new(SpyHandler(handler.clone())));

    bus.publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
        container_names: vec![],
        success: false,
    }));

    let calls = handler.called_with.borrow();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].is_empty());
}

/// Wrapper that delegates to an Rc<SpyCleanupHandler> so we can
/// inspect the handler after moving it into the event bus.
struct SpyHandler(Rc<SpyCleanupHandler>);

impl ContainerCleanupUseCase for SpyHandler {
    fn handle_act_run_completed(&self, container_names: &[String]) {
        self.0.handle_act_run_completed(container_names);
    }
}
