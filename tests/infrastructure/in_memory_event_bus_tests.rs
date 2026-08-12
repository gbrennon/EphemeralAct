use std::{cell::RefCell, rc::Rc};

use ephemeral_act::{
    core::{
        dtos::ContainerCleanupRequest,
        events::{ActRunCompletedPayload, DomainEvent},
        ports::{
            inbound::container_cleanup_port::ContainerCleanupPort,
            outbound::event_publisher::EventPublisherPort,
        },
    },
    infrastructure::InMemoryEventBus,
};

#[cfg(test)]
mod tests {
    use super::*;

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

    impl ContainerCleanupPort for SpyCleanupHandler {
        fn execute(&self, request: ContainerCleanupRequest) {
            self.called_with.borrow_mut().push(request.container_names);
        }
    }

    struct SpyHandler(Rc<SpyCleanupHandler>);

    impl ContainerCleanupPort for SpyHandler {
        fn execute(&self, request: ContainerCleanupRequest) {
            self.0.execute(request);
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
}
