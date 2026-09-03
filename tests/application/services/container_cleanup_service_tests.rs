use std::sync::Arc;

use ephact::application::{
    dtos::ContainerCleanupRequest, ports::inbound::container_cleanup_port::ContainerCleanupPort,
    services::container_cleanup_service::ContainerCleanupService,
};

use crate::common::fakes::{
    spy_container_runtime::SpyContainerRuntime,
    stub_failing_container_runtime::StubFailingContainerRuntime,
};

#[test]
fn execute_stops_and_removes_each_requested_container() {
    let runtime = SpyContainerRuntime::new();
    let service = ContainerCleanupService::new(Arc::new(runtime.clone()));
    let request = ContainerCleanupRequest::new(vec!["app1".into(), "app2".into()]);

    service.execute(request);

    assert_eq!(runtime.stopped_containers(), vec!["app1", "app2"]);
    assert_eq!(runtime.removed_containers(), vec!["app1", "app2"]);
}

#[test]
fn execute_with_empty_request_does_not_stop_or_remove_containers() {
    let runtime = SpyContainerRuntime::new();
    let service = ContainerCleanupService::new(Arc::new(runtime.clone()));
    let request = ContainerCleanupRequest::default();

    service.execute(request);

    assert!(runtime.stopped_containers().is_empty());
    assert!(runtime.removed_containers().is_empty());
}

#[test]
fn execute_continues_when_runtime_fails_to_stop_or_remove() {
    let runtime = StubFailingContainerRuntime;
    let service = ContainerCleanupService::new(Arc::new(runtime));
    let request = ContainerCleanupRequest::new(vec!["app1".into(), "app2".into()]);

    service.execute(request);
}
