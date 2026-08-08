use std::collections::HashMap;

use ephemeral_act::{
    core::ports::outbound::{ContainerConfig, ContainerError, ContainerRuntime, FileEntry},
    infrastructure::runners::podman_runtime::PodmanRuntime,
};

fn podman_available() -> bool {
    PodmanRuntime::new().is_ok()
}

#[test]
fn new_connects_to_podman_socket() {
    let runtime = PodmanRuntime::new();
    assert!(runtime.is_ok(), "Podman should be reachable");
}

#[test]
fn get_host_info_returns_valid_data() {
    let runtime = PodmanRuntime::new().expect("Podman should be reachable");
    let info = runtime
        .get_host_info()
        .expect("Host info should be retrievable");
    assert!(!info.os.is_empty(), "OS should not be empty");
    assert!(!info.arch.is_empty(), "Arch should not be empty");
    assert!(
        !info.engine_version.is_empty(),
        "Engine version should not be empty"
    );
}

#[test]
fn pull_image_alpine_succeeds() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    let result = runtime.pull_image("alpine:latest", None);
    assert!(result.is_ok(), "Failed to pull alpine:latest: {:?}", result);
}

#[test]
fn pull_image_with_platform_succeeds() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    let result = runtime.pull_image("alpine:latest", Some("linux/amd64"));
    assert!(
        result.is_ok(),
        "Failed to pull alpine:latest with platform: {:?}",
        result
    );
}

#[test]
fn pull_nonexistent_image_fails() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    let result = runtime.pull_image("nonexistent/image:fake", None);
    assert!(
        matches!(result, Err(ContainerError::ImagePullFailed(_, _))),
        "Expected ImagePullFailed, got: {:?}",
        result
    );
}

#[test]
fn create_and_remove_container_lifecycle() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    runtime
        .pull_image("alpine:latest", None)
        .expect("Should pull alpine");

    let config = ContainerConfig {
        image: "alpine:latest".to_string(),
        platform: None,
        env: HashMap::from([("TEST_VAR".to_string(), "test_value".to_string())]),
        binds: Vec::new(),
        workdir: None,
        cmd: Some(vec!["sleep".to_string(), "10".to_string()]),
        entrypoint: None,
        network: None,
        name: Some("ephemeral_act_test_lifecycle".to_string()),
    };

    let container = runtime
        .create_container(&config)
        .expect("Should create container");

    container.remove().expect("Should remove container");
}

#[test]
fn exec_echo_returns_stdout() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    runtime
        .pull_image("alpine:latest", None)
        .expect("Should pull alpine");

    let config = ContainerConfig {
        image: "alpine:latest".to_string(),
        platform: None,
        env: HashMap::new(),
        binds: Vec::new(),
        workdir: None,
        cmd: Some(vec!["sleep".to_string(), "30".to_string()]),
        entrypoint: None,
        network: None,
        name: Some("ephemeral_act_test_exec".to_string()),
    };

    let container = runtime
        .create_container(&config)
        .expect("Should create container");

    let result = container
        .exec(
            &["echo".to_string(), "hello".to_string()],
            None,
            &HashMap::new(),
        )
        .expect("Should exec echo");

    assert!(result.stdout.contains("hello"), "stdout: {}", result.stdout);
    assert!(result.stderr.is_empty(), "stderr: {}", result.stderr);

    container.remove().expect("Should remove container");
}

#[test]
fn exec_with_workdir_runs_in_specified_directory() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    runtime
        .pull_image("alpine:latest", None)
        .expect("Should pull alpine");

    let config = ContainerConfig {
        image: "alpine:latest".to_string(),
        platform: None,
        env: HashMap::new(),
        binds: Vec::new(),
        workdir: None,
        cmd: Some(vec!["sleep".to_string(), "30".to_string()]),
        entrypoint: None,
        network: None,
        name: Some("ephemeral_act_test_workdir".to_string()),
    };

    let container = runtime
        .create_container(&config)
        .expect("Should create container");

    let result = container
        .exec(&["pwd".to_string()], Some("/tmp"), &HashMap::new())
        .expect("Should exec pwd");

    assert!(
        result.stdout.trim() == "/tmp",
        "Expected /tmp, got: {}",
        result.stdout
    );

    container.remove().expect("Should remove container");
}

#[test]
fn copy_to_and_copy_from_roundtrip() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    runtime
        .pull_image("alpine:latest", None)
        .expect("Should pull alpine");

    let config = ContainerConfig {
        image: "alpine:latest".to_string(),
        platform: None,
        env: HashMap::new(),
        binds: Vec::new(),
        workdir: None,
        cmd: Some(vec!["sleep".to_string(), "30".to_string()]),
        entrypoint: None,
        network: None,
        name: Some("ephemeral_act_test_copy".to_string()),
    };

    let container = runtime
        .create_container(&config)
        .expect("Should create container");

    let entries = vec![FileEntry {
        path: "test.txt".to_string(),
        content: b"hello from test".to_vec(),
        mode: 0o644,
    }];

    container
        .copy_to("/tmp", &entries)
        .expect("Should copy file to container");

    let retrieved = container
        .copy_from("/tmp/test.txt")
        .expect("Should copy file from container");

    assert!(!retrieved.is_empty(), "Should have at least one entry");
    assert_eq!(
        retrieved[0].path, "test.txt",
        "Path should be test.txt, got: {}",
        retrieved[0].path
    );
    assert_eq!(
        retrieved[0].content,
        b"hello from test".to_vec(),
        "Content mismatch"
    );

    container.remove().expect("Should remove container");
}

#[test]
fn get_runner_context_returns_expected_paths() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    runtime
        .pull_image("alpine:latest", None)
        .expect("Should pull alpine");

    let config = ContainerConfig {
        image: "alpine:latest".to_string(),
        platform: None,
        env: HashMap::from([("GITHUB_ENV".to_string(), "/tmp/env".to_string())]),
        binds: Vec::new(),
        workdir: None,
        cmd: Some(vec!["sleep".to_string(), "10".to_string()]),
        entrypoint: None,
        network: None,
        name: Some("ephemeral_act_test_context".to_string()),
    };

    let container = runtime
        .create_container(&config)
        .expect("Should create container");

    let ctx = container
        .get_runner_context()
        .expect("Should get runner context");

    assert_eq!(ctx.workspace, "/github/workspace");
    assert_eq!(ctx.home, "/github/home");
    assert_eq!(ctx.action_path, "/github/action");
    assert_eq!(ctx.temp, "/github/temp");
    assert_eq!(ctx.tool_cache, "/github/tool_cache");
    assert_eq!(ctx.env.get("GITHUB_ENV"), Some(&"/tmp/env".to_string()));

    container.remove().expect("Should remove container");
}

#[test]
fn remove_nonexistent_container_fails() {
    if !podman_available() {
        return;
    }
    let runtime = PodmanRuntime::new().unwrap();
    runtime
        .pull_image("alpine:latest", None)
        .expect("Should pull alpine");

    let config = ContainerConfig {
        image: "alpine:latest".to_string(),
        platform: None,
        env: HashMap::new(),
        binds: Vec::new(),
        workdir: None,
        cmd: Some(vec!["sleep".to_string(), "10".to_string()]),
        entrypoint: None,
        network: None,
        name: Some("ephemeral_act_test_remove".to_string()),
    };

    let container = runtime
        .create_container(&config)
        .expect("Should create container");

    container.remove().expect("Should remove container");

    let second_remove = container.remove();
    assert!(
        matches!(second_remove, Err(ContainerError::RemovalFailed(_, _))),
        "Expected RemovalFailed on second remove, got: {:?}",
        second_remove
    );
}
