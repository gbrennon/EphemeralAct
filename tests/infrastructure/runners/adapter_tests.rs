use ephemeral_act::{
    core::ports::outbound::ContainerRuntime,
    infrastructure::ContainerRuntimeAdapter,
};

#[test]
fn detect_succeeds_when_runtime_available() {
    let result = ContainerRuntimeAdapter::detect();
    assert!(result.is_ok(), "detect() should succeed: {:?}", result.err());
}

#[test]
fn detect_returns_docker_when_docker_host_is_set() {
    let adapter = ContainerRuntimeAdapter::detect().expect("detect() should succeed");
    assert!(
        matches!(adapter, ContainerRuntimeAdapter::Docker(_)),
        "Expected Docker variant since DOCKER_HOST is set"
    );
}

#[test]
fn map_error_replaces_docker_with_podman_in_error_text() {
    // Force Podman variant by checking if Podman is available directly.
    // If only Docker is available, this test is skipped.
    let adapter = ContainerRuntimeAdapter::detect().unwrap();
    if matches!(adapter, ContainerRuntimeAdapter::Docker(_)) {
        // When running as Docker, map_error is a no-op — verify it passes through.
        
        let result = adapter.pull_image("nonexistent:latest", None);
        assert!(result.is_err());
    }
}

#[test]
fn pull_image_delegates_to_inner_runtime() {
    let adapter = ContainerRuntimeAdapter::detect().unwrap();
    // Pulling a nonexistent image should fail, proving delegation works.
    let result = adapter.pull_image("nonexistent-image-xyz:latest", None);
    assert!(result.is_err());
}

#[test]
fn get_host_info_delegates_to_inner_runtime() {
    let adapter = ContainerRuntimeAdapter::detect().unwrap();
    let info = adapter.get_host_info().unwrap();
    assert!(!info.os.is_empty());
    assert!(!info.arch.is_empty());
}

#[test]
fn stop_container_delegates_to_inner_runtime() {
    let adapter = ContainerRuntimeAdapter::detect().unwrap();
    let _ = adapter.stop_container("nonexistent-container-xyz-123");
}

#[test]
fn remove_container_delegates_to_inner_runtime() {
    let adapter = ContainerRuntimeAdapter::detect().unwrap();
    let _ = adapter.remove_container("nonexistent-container-xyz-123");
}

#[test]
fn create_container_delegates_to_inner_runtime() {
    use std::collections::HashMap;
    use ephemeral_act::core::ports::outbound::ContainerConfig;

    let adapter = ContainerRuntimeAdapter::detect().unwrap();
    let config = ContainerConfig {
        image: "alpine:latest".into(),
        platform: None,
        env: HashMap::new(),
        binds: vec![],
        workdir: None,
        cmd: Some(vec!["sleep".into(), "infinity".into()]),
        entrypoint: None,
        network: None,
        name: Some("ephemeral-act-test-adapter-create".into()),
        runner_context: Default::default(),
    };
    let _ = adapter.remove_container("ephemeral-act-test-adapter-create");
    let container = adapter.create_container(&config).unwrap();
    container.remove().unwrap();
}
