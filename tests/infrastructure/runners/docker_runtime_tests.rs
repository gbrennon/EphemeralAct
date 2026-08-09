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

#[test]
fn pull_image_with_platform_succeeds() {
    let runtime = DockerRuntime::new().unwrap();
    // alpine is small and widely available
    let result = runtime.pull_image("alpine:latest", Some("linux/amd64"));
    assert!(result.is_ok());
}

#[test]
fn stop_running_container_succeeds() {
    let runtime = DockerRuntime::new().unwrap();
    let config = make_config("ephemeral-act-test-docker-stop");
    let _ = runtime.remove_container("ephemeral-act-test-docker-stop");
    let container = runtime.create_container(&config).unwrap();
    runtime.stop_container("ephemeral-act-test-docker-stop").unwrap();
    container.remove().unwrap();
}

#[test]
fn exec_with_workdir_runs_in_specified_directory() {
    let runtime = DockerRuntime::new().unwrap();
    let config = make_config("ephemeral-act-test-docker-workdir");
    let _ = runtime.remove_container("ephemeral-act-test-docker-workdir");
    let container = runtime.create_container(&config).unwrap();
    let result = container
        .exec(&["pwd".into()], Some("/tmp"), &HashMap::new())
        .unwrap();
    assert_eq!(result.stdout.trim(), "/tmp");
    assert_eq!(result.exit_code, 0);
    container.remove().unwrap();
}

#[test]
fn exec_with_env_passes_environment() {
    let runtime = DockerRuntime::new().unwrap();
    let config = make_config("ephemeral-act-test-docker-env");
    let _ = runtime.remove_container("ephemeral-act-test-docker-env");
    let container = runtime.create_container(&config).unwrap();
    let mut env = HashMap::new();
    env.insert("MY_VAR".into(), "my_value".into());
    let result = container
        .exec(&["sh".into(), "-c".into(), "echo -n $MY_VAR".into()], None, &env)
        .unwrap();
    assert_eq!(result.stdout, "my_value");
    assert_eq!(result.exit_code, 0);
    container.remove().unwrap();
}

#[test]
fn get_runner_context_returns_expected_paths() {
    let runtime = DockerRuntime::new().unwrap();
    let config = make_config("ephemeral-act-test-docker-context");
    let _ = runtime.remove_container("ephemeral-act-test-docker-context");
    let container = runtime.create_container(&config).unwrap();
    let ctx = container.get_runner_context().unwrap();
    assert_eq!(ctx.workspace, "/workspace");
    assert_eq!(ctx.home, "/home");
    container.remove().unwrap();
}
