use ephemeral_act::{
    core::ports::outbound::ImageMapper,
    infrastructure::{ContainerRuntimeAdapter, PlatformImageMapper},
};

#[test]
fn map_platform_ubuntu_latest_returns_act_latest() {
    assert_eq!(
        PlatformImageMapper.map("ubuntu-latest"),
        "catthehacker/ubuntu:act-latest"
    );
}

#[test]
fn map_platform_ubuntu_24_04_returns_act_24_04() {
    assert_eq!(
        PlatformImageMapper.map("ubuntu-24.04"),
        "catthehacker/ubuntu:act-24.04"
    );
}

#[test]
fn map_platform_ubuntu_22_04_returns_act_22_04() {
    assert_eq!(
        PlatformImageMapper.map("ubuntu-22.04"),
        "catthehacker/ubuntu:act-22.04"
    );
}

#[test]
fn map_platform_ubuntu_20_04_returns_act_20_04() {
    assert_eq!(
        PlatformImageMapper.map("ubuntu-20.04"),
        "catthehacker/ubuntu:act-20.04"
    );
}

#[test]
fn map_platform_unknown_returns_input_unchanged() {
    assert_eq!(
        PlatformImageMapper.map("custom-image:latest"),
        "custom-image:latest"
    );
}

#[test]
fn map_platform_empty_string_returns_empty() {
    assert_eq!(PlatformImageMapper.map(""), "");
}

#[test]
fn detect_succeeds() {
    let result = ContainerRuntimeAdapter::detect();
    assert!(
        result.is_ok(),
        "detect() should succeed when a runtime is available"
    );
}

#[test]
fn detect_returns_docker_when_docker_host_is_set() {
    let adapter = ContainerRuntimeAdapter::detect().expect("detect() should succeed");
    assert!(
        matches!(adapter, ContainerRuntimeAdapter::Docker(_)),
        "Expected Docker variant since DOCKER_HOST is set to Podman socket"
    );
}
