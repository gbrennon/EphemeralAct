use ephact::{
    application::ports::outbound::run_node_action_port::RunNodeActionPort,
    infrastructure::actions::run_node_action_service::RunNodeActionService,
};
use std::{collections::HashMap, path::Path};

use crate::common::fakes::{
    fake_build_action_input_environment_port::FakeBuildActionInputEnvironmentPort,
    fake_copy_action_to_container_port::FakeCopyActionToContainerPort,
    fake_resolve_node_binary_port::FakeResolveNodeBinaryPort,
    stub_failing_container::StubFailingContainer, stub_recording_container::StubRecordingContainer,
};
use ephact::application::dtos::RunNodeActionRequest;

fn action_env() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("INPUT_KEY".to_string(), "build-cache".to_string());
    env
}

fn service(copier: FakeCopyActionToContainerPort) -> RunNodeActionService {
    RunNodeActionService::new(
        Box::new(copier),
        Box::new(FakeBuildActionInputEnvironmentPort::returning(action_env())),
        Box::new(FakeResolveNodeBinaryPort::returning("/usr/bin/node")),
    )
}

#[test]
fn execute_runs_the_entry_point_with_the_resolved_binary() {
    let container = StubRecordingContainer::new();

    service(FakeCopyActionToContainerPort::returning(
        "/tmp/actions/cache",
    ))
    .execute(RunNodeActionRequest {
        action_dir: Path::new("/repo/actions/cache"),
        entry_point: "dist/index.js",
        inputs: &HashMap::new(),
        env: &HashMap::new(),
        container: &container,
    })
    .unwrap();

    assert_eq!(
        container.executed_commands()[0],
        vec![
            "/usr/bin/node".to_string(),
            "/tmp/actions/cache/dist/index.js".to_string()
        ]
    );
}

#[test]
fn execute_passes_the_built_environment_to_the_container() {
    let container = StubRecordingContainer::new();

    service(FakeCopyActionToContainerPort::returning(
        "/tmp/actions/cache",
    ))
    .execute(RunNodeActionRequest {
        action_dir: Path::new("/repo/actions/cache"),
        entry_point: "dist/index.js",
        inputs: &HashMap::new(),
        env: &HashMap::new(),
        container: &container,
    })
    .unwrap();

    assert_eq!(
        container.exec_environments()[0]
            .get("INPUT_KEY")
            .map(String::as_str),
        Some("build-cache")
    );
}

#[test]
fn execute_propagates_a_copy_failure() {
    let error = service(FakeCopyActionToContainerPort::failing(
        "failed to copy action files",
    ))
    .execute(RunNodeActionRequest {
        action_dir: Path::new("/repo/actions/cache"),
        entry_point: "dist/index.js",
        inputs: &HashMap::new(),
        env: &HashMap::new(),
        container: &StubRecordingContainer::new(),
    })
    .unwrap_err();

    assert_eq!(error.message, "failed to copy action files");
}

#[test]
fn execute_reports_a_failing_entry_point() {
    let error = service(FakeCopyActionToContainerPort::returning(
        "/tmp/actions/cache",
    ))
    .execute(RunNodeActionRequest {
        action_dir: Path::new("/repo/actions/cache"),
        entry_point: "dist/index.js",
        inputs: &HashMap::new(),
        env: &HashMap::new(),
        container: &StubFailingContainer,
    })
    .unwrap_err();

    assert!(
        error.message.starts_with("failed to run node action"),
        "{}",
        error.message
    );
}
