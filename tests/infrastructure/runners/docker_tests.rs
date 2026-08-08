use ephemeral_act::{
    core::ports::outbound::ContainerRuntime, infrastructure::runners::docker_runtime::DockerRuntime,
};

#[test]
fn new_succeeds_when_docker_host_points_to_podman() {
    let runtime = DockerRuntime::new();
    assert!(
        runtime.is_ok(),
        "DockerRuntime::new() should succeed when DOCKER_HOST points to Podman socket"
    );
}

#[test]
fn get_host_info_returns_valid_data_via_podman_socket() {
    let runtime = DockerRuntime::new().expect("DockerRuntime::new() should succeed");
    let info = runtime
        .get_host_info()
        .expect("get_host_info() should succeed via Podman socket");
    assert!(!info.os.is_empty(), "OS should not be empty");
    assert!(!info.arch.is_empty(), "Arch should not be empty");
}
