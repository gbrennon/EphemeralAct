use std::sync::Arc;

use ephact::infrastructure::{
    ContainerRuntimeAdapter,
    containers::{ContainerConfig, ContainerRuntimePort, DockerRuntime, PodmanRuntime},
};

#[cfg(test)]
mod tests {
    use super::*;

    fn try_podman_adapter() -> Option<ContainerRuntimeAdapter> {
        PodmanRuntime::new()
            .ok()
            .map(ContainerRuntimeAdapter::Podman)
    }

    fn try_docker_adapter() -> Option<ContainerRuntimeAdapter> {
        DockerRuntime::new()
            .ok()
            .map(ContainerRuntimeAdapter::Docker)
    }

    macro_rules! adapter {
        () => {
            match ContainerRuntimeAdapter::detect() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("SKIP: no container runtime available: {e:?}");
                    return;
                }
            }
        };
    }

    #[test]
    fn detect_succeeds_when_runtime_available() {
        let result = ContainerRuntimeAdapter::detect();
        if let Err(e) = &result {
            eprintln!("SKIP: no container runtime available: {e:?}");
            return;
        }
        assert!(result.is_ok(), "detect() should succeed");
    }

    #[test]
    fn detect_returns_docker_when_docker_host_is_set() {
        if try_docker_adapter().is_none() {
            eprintln!("SKIP: Docker runtime not available on this host");
            return;
        }
        let adapter = match ContainerRuntimeAdapter::detect() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("SKIP: no container runtime available: {e:?}");
                return;
            }
        };
        assert!(
            matches!(adapter, ContainerRuntimeAdapter::Docker(_)),
            "Expected Docker variant since Docker is available"
        );
    }

    #[test]
    fn map_error_is_noop_for_docker_variant() {
        let adapter = match try_docker_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: Docker runtime not available on this host");
                return;
            }
        };
        let result = adapter.pull_image("nonexistent-image-xyz:latest", None);
        assert!(result.is_err());
        let err_text = format!("{:?}", result.unwrap_err());
        assert!(
            err_text.contains("Docker"),
            "Docker variant should preserve 'Docker' in error: {err_text}"
        );
    }

    #[test]
    fn map_error_replaces_docker_with_podman_in_error_text() {
        let podman = match PodmanRuntime::new() {
            Ok(rt) => rt,
            Err(_) => {
                eprintln!("SKIP: PodmanRuntime not available on this host");
                return;
            }
        };
        let adapter = ContainerRuntimeAdapter::Podman(podman);

        let result = adapter.pull_image("nonexistent-image-xyz:latest", None);
        assert!(result.is_err());
        let err_text = format!("{:?}", result.unwrap_err());
        assert!(
            !err_text.contains("Docker"),
            "Podman variant should NOT contain 'Docker' in error: {err_text}"
        );
        assert!(
            err_text.contains("Podman"),
            "Podman variant should contain 'Podman' in error: {err_text}"
        );
    }

    #[test]
    fn pull_image_delegates_to_inner_runtime() {
        let adapter = adapter!();
        let result = adapter.pull_image("nonexistent-image-xyz:latest", None);
        assert!(result.is_err());
    }

    #[test]
    fn get_host_info_delegates_to_inner_runtime() {
        let adapter = adapter!();
        let info = adapter.get_host_info().unwrap();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn stop_container_delegates_to_inner_runtime() {
        let adapter = adapter!();
        let _ = adapter.stop_container("nonexistent-container-xyz-123");
    }

    #[test]
    fn remove_container_delegates_to_inner_runtime() {
        let adapter = adapter!();
        let _ = adapter.remove_container("nonexistent-container-xyz-123");
    }

    #[test]
    fn create_container_delegates_to_inner_runtime() {
        use std::collections::HashMap;

        let adapter = adapter!();
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

    #[test]
    fn podman_variant_pull_image_delegates() {
        let adapter = match try_podman_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let result = adapter.pull_image("nonexistent-image-xyz:latest", None);
        assert!(result.is_err());
    }

    #[test]
    fn podman_variant_get_host_info_delegates() {
        let adapter = match try_podman_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let info = adapter.get_host_info().unwrap();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn podman_variant_stop_container_delegates() {
        let adapter = match try_podman_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let _ = adapter.stop_container("nonexistent-container-xyz-123");
    }

    #[test]
    fn podman_variant_remove_container_delegates() {
        let adapter = match try_podman_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let _ = adapter.remove_container("nonexistent-container-xyz-123");
    }

    #[test]
    fn podman_variant_create_container_delegates() {
        use std::collections::HashMap;

        let adapter = match try_podman_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let config = ContainerConfig {
            image: "alpine:latest".into(),
            platform: None,
            env: HashMap::new(),
            binds: vec![],
            workdir: None,
            cmd: Some(vec!["sleep".into(), "infinity".into()]),
            entrypoint: None,
            network: None,
            name: Some("ephemeral-act-test-adapter-podman-create-465968".into()),
            runner_context: Default::default(),
        };
        let _ = adapter.remove_container("ephemeral-act-test-adapter-podman-create-465968");
        let container = adapter.create_container(&config).unwrap();
        container.remove().unwrap();
    }

    #[test]
    fn podman_variant_map_error_on_stop_container() {
        let adapter = match try_podman_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let result = adapter.stop_container("nonexistent-container-xyz-123");
        if let Err(e) = result {
            let text = format!("{:?}", e);
            assert!(
                !text.contains("Docker"),
                "Podman error should not contain 'Docker': {text}"
            );
        }
    }

    #[test]
    fn podman_variant_map_error_on_remove_container() {
        let adapter = match try_podman_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let result = adapter.remove_container("nonexistent-container-xyz-123");
        if let Err(e) = result {
            let text = format!("{:?}", e);
            assert!(
                !text.contains("Docker"),
                "Podman error should not contain 'Docker': {text}"
            );
        }
    }

    #[test]
    fn podman_variant_map_error_on_create_container() {
        let adapter = match try_podman_adapter() {
            Some(a) => a,
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let config = ContainerConfig {
            image: "nonexistent-image-xyz:latest".into(),
            platform: None,
            env: std::collections::HashMap::new(),
            binds: vec![],
            workdir: None,
            cmd: Some(vec!["echo".into(), "hello".into()]),
            entrypoint: None,
            network: None,
            name: Some("ephemeral-act-test-podman-map-error".into()),
            runner_context: Default::default(),
        };
        let result = adapter.create_container(&config);
        if let Err(e) = result {
            let text = format!("{:?}", e);
            assert!(
                !text.contains("Docker"),
                "Podman error should not contain 'Docker': {text}"
            );
        }
    }

    #[test]
    fn arc_podman_variant_get_host_info() {
        let adapter = match try_podman_adapter() {
            Some(a) => Arc::new(a),
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let info = adapter.get_host_info().unwrap();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn arc_podman_variant_pull_image_error_mapped() {
        let adapter = match try_podman_adapter() {
            Some(a) => Arc::new(a),
            None => {
                eprintln!("SKIP: PodmanRuntime not available");
                return;
            }
        };
        let result = adapter.pull_image("nonexistent-image-xyz:latest", None);
        assert!(result.is_err());
        let err_text = format!("{:?}", result.unwrap_err());
        assert!(
            !err_text.contains("Docker"),
            "Arc+Podman error should not contain 'Docker': {err_text}"
        );
    }
}
