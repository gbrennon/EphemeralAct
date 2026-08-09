use std::collections::HashMap;

use ephemeral_act::{
    core::ports::outbound::{ContainerConfig, ContainerRuntime},
    infrastructure::runners::DockerRuntime,
};

fn make_config(name: &str) -> ContainerConfig {
    ContainerConfig {
        image: "alpine:latest".into(),
        platform: None,
        env: HashMap::new(),
        binds: vec![],
        workdir: None,
        cmd: Some(vec!["sleep".into(), "infinity".into()]),
        entrypoint: None,
        network: None,
        name: Some(name.into()),
        runner_context: Default::default(),
    }
}

#[test]
fn new_connects_to_socket() {
    assert!(DockerRuntime::new().is_ok());
}

#[test]
fn get_host_info_returns_valid_data() {
    let runtime = DockerRuntime::new().unwrap();
    let info = runtime.get_host_info().unwrap();
    assert!(!info.os.is_empty());
    assert!(!info.arch.is_empty());
}

#[test]
fn pull_nonexistent_image_fails() {
    let runtime = DockerRuntime::new().unwrap();
    assert!(runtime.pull_image("nonexistent-image-xyz-123:latest", None).is_err());
}

#[test]
fn create_and_remove_container_lifecycle() {
    let runtime = DockerRuntime::new().unwrap();
    let config = make_config("ephemeral-act-test-docker-lifecycle");
    let _ = runtime.remove_container("ephemeral-act-test-docker-lifecycle");
    let container = runtime.create_container(&config).unwrap();
    container.remove().unwrap();
}

#[test]
fn stop_nonexistent_container_is_noop() {
    let runtime = DockerRuntime::new().unwrap();
    let _ = runtime.stop_container("nonexistent-container-xyz-123");
}

#[test]
fn remove_nonexistent_container_is_noop() {
    let runtime = DockerRuntime::new().unwrap();
    let _ = runtime.remove_container("nonexistent-container-xyz-123");
}

#[test]
fn exec_echo_returns_stdout() {
    let runtime = DockerRuntime::new().unwrap();
    let config = make_config("ephemeral-act-test-docker-exec");
    let _ = runtime.remove_container("ephemeral-act-test-docker-exec");
    let container = runtime.create_container(&config).unwrap();
    let result = container
        .exec(&["echo".into(), "-n".into(), "hello".into()], None, &HashMap::new())
        .unwrap();
    assert_eq!(result.stdout, "hello");
    assert_eq!(result.exit_code, 0);
    container.remove().unwrap();
}
