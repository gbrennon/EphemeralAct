mod common;

use std::collections::HashMap;

use ephemeral_act::core::ports::outbound::{ContainerConfig, ContainerRuntime, RunnerContext};

use crate::common::FakeRuntime;

// ---------------------------------------------------------------------------
// pull_image
// ---------------------------------------------------------------------------

#[test]
fn pull_image_records_image_name() {
    let runtime = FakeRuntime::new();
    runtime.pull_image("alpine:latest", None).unwrap();
    assert_eq!(*runtime.pulled_images.borrow(), vec!["alpine:latest"]);
}

#[test]
fn pull_image_with_platform_still_records() {
    let runtime = FakeRuntime::new();
    runtime
        .pull_image("alpine:latest", Some("linux/amd64"))
        .unwrap();
    assert_eq!(*runtime.pulled_images.borrow(), vec!["alpine:latest"]);
}

// ---------------------------------------------------------------------------
// create_container
// ---------------------------------------------------------------------------

#[test]
fn create_container_records_config() {
    let runtime = FakeRuntime::new();
    let config = ContainerConfig {
        image: "alpine:latest".into(),
        platform: None,
        env: HashMap::new(),
        binds: vec![],
        workdir: Some("/work".into()),
        cmd: Some(vec!["sleep".into(), "infinity".into()]),
        entrypoint: None,
        network: None,
        name: Some("test-container".into()),
        runner_context: RunnerContext::default(),
    };
    runtime.create_container(&config).unwrap();
    let created = &runtime.created_containers.borrow();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].image, "alpine:latest");
    assert_eq!(created[0].name.as_deref(), Some("test-container"));
}

// ---------------------------------------------------------------------------
// exec (via FakeContainerHandle)
// ---------------------------------------------------------------------------

#[test]
fn exec_returns_preloaded_result() {
    let runtime = FakeRuntime::new();
    runtime
        .exec_results
        .borrow_mut()
        .push(ephemeral_act::core::ports::outbound::ExecResult {
            exit_code: 0,
            stdout: "hello".into(),
            stderr: String::new(),
        });
    let config = ContainerConfig {
        image: "alpine:latest".into(),
        platform: None,
        env: HashMap::new(),
        binds: vec![],
        workdir: None,
        cmd: Some(vec!["echo".into(), "hello".into()]),
        entrypoint: None,
        network: None,
        name: None,
        runner_context: RunnerContext::default(),
    };
    let container = runtime.create_container(&config).unwrap();
    let result = container
        .exec(&["echo".into(), "hello".into()], None, &HashMap::new())
        .unwrap();
    assert_eq!(result.stdout, "hello");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn exec_returns_default_success_when_no_preloaded_results() {
    let runtime = FakeRuntime::new();
    let config = ContainerConfig {
        image: "alpine:latest".into(),
        platform: None,
        env: HashMap::new(),
        binds: vec![],
        workdir: None,
        cmd: Some(vec!["echo".into(), "hello".into()]),
        entrypoint: None,
        network: None,
        name: None,
        runner_context: RunnerContext::default(),
    };
    let container = runtime.create_container(&config).unwrap();
    let result = container.exec(&["echo".into()], None, &HashMap::new());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().exit_code, 0);
}

// ---------------------------------------------------------------------------
// stop_container / remove_container
// ---------------------------------------------------------------------------

#[test]
fn stop_container_records_name() {
    let runtime = FakeRuntime::new();
    runtime.stop_container("my-container").unwrap();
    assert_eq!(*runtime.stopped_containers.borrow(), vec!["my-container"]);
}

#[test]
fn remove_container_records_name() {
    let runtime = FakeRuntime::new();
    runtime.remove_container("my-container").unwrap();
    assert_eq!(*runtime.removed_containers.borrow(), vec!["my-container"]);
}

// ---------------------------------------------------------------------------
// get_host_info
// ---------------------------------------------------------------------------

#[test]
fn get_host_info_returns_fake_data() {
    let runtime = FakeRuntime::new();
    let info = runtime.get_host_info().unwrap();
    assert_eq!(info.os, "linux");
    assert_eq!(info.arch, "amd64");
}
