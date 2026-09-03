use ephact::application::{
    dtos::ResolveNodeBinaryRequest,
    ports::outbound::resolve_node_binary_port::ResolveNodeBinaryPort,
    services::resolve_node_binary_service::ResolveNodeBinaryService,
};

use crate::common::fakes::{
    stub_failing_container::StubFailingContainer, stub_scripted_container::StubScriptedContainer,
};

#[test]
fn execute_returns_the_trimmed_path_a_successful_lookup_reports() {
    let container = StubScriptedContainer::answering(0, "/opt/toolcache/node/bin/node\n");

    let binary = ResolveNodeBinaryService::new().execute(ResolveNodeBinaryRequest {
        container: &container,
    });

    assert_eq!(binary, "/opt/toolcache/node/bin/node");
}

#[test]
fn execute_falls_back_to_node_when_the_lookup_exits_non_zero() {
    let container = StubScriptedContainer::answering(1, "/opt/toolcache/node/bin/node\n");

    let binary = ResolveNodeBinaryService::new().execute(ResolveNodeBinaryRequest {
        container: &container,
    });

    assert_eq!(binary, "node");
}

#[test]
fn execute_falls_back_to_node_when_the_lookup_reports_nothing() {
    let container = StubScriptedContainer::answering(0, "   \n");

    let binary = ResolveNodeBinaryService::new().execute(ResolveNodeBinaryRequest {
        container: &container,
    });

    assert_eq!(binary, "node");
}

#[test]
fn execute_falls_back_to_node_when_the_lookup_fails() {
    let binary = ResolveNodeBinaryService::new().execute(ResolveNodeBinaryRequest {
        container: &StubFailingContainer,
    });

    assert_eq!(binary, "node");
}
