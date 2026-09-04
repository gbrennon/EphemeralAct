use std::sync::Arc;

use ephact::{
    application::ports::outbound::EventBusPort,
    domain::events::{ActRunCompletedPayload, DomainEvent},
    infrastructure::{containers::ContainerCleanupHandler, messaging::InMemoryEventBus},
};

use crate::common::fakes::fake_runtime::FakeRuntime;

#[test]
fn publish_act_run_completed_stops_and_removes_containers() {
    let runtime = Arc::new(FakeRuntime::new());
    let cleanup_handler = Box::new(ContainerCleanupHandler::new(runtime.clone()));
    let bus = InMemoryEventBus::new(cleanup_handler);

    let event = DomainEvent::ActRunCompleted(ActRunCompletedPayload {
        container_names: vec!["container-a".into(), "container-b".into()],
        success: true,
    });

    bus.publish(event);

    assert_eq!(
        *runtime.stopped_containers.lock(),
        vec!["container-a".to_string(), "container-b".to_string()]
    );
    assert_eq!(
        *runtime.removed_containers.lock(),
        vec!["container-a".to_string(), "container-b".to_string()]
    );
}
