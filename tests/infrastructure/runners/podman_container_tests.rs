use std::collections::HashMap;

use ephact::{
    application::ports::outbound::{ContainerConfig, ContainerRuntimePort, FileEntry},
    infrastructure::runners::PodmanRuntime,
};

#[cfg(test)]
mod tests {
    use super::*;

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

    macro_rules! runtime {
        () => {
            match PodmanRuntime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("SKIP: Podman runtime not available: {e:?}");
                    return;
                }
            }
        };
    }

    #[test]
    fn exec_echo_returns_stdout_and_zero_exit() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-podman-ct-exec");
        let _ = runtime.remove_container("ephemeral-act-test-podman-ct-exec");
        let container = runtime.create_container(&config).unwrap();

        let result = container
            .exec(
                &["echo".into(), "-n".into(), "hello".into()],
                None,
                &HashMap::new(),
            )
            .unwrap();

        assert_eq!(result.stdout, "hello");
        assert_eq!(result.exit_code, 0);
        container.remove().unwrap();
    }

    #[test]
    fn exec_failing_command_returns_nonzero_exit() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-podman-ct-exitcode");
        let _ = runtime.remove_container("ephemeral-act-test-podman-ct-exitcode");
        let container = runtime.create_container(&config).unwrap();

        let result = container
            .exec(
                &["sh".into(), "-c".into(), "exit 42".into()],
                None,
                &HashMap::new(),
            )
            .unwrap();

        assert_eq!(result.exit_code, 42);
        container.remove().unwrap();
    }

    #[test]
    fn exec_with_env_passes_environment() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-podman-ct-env");
        let _ = runtime.remove_container("ephemeral-act-test-podman-ct-env");
        let container = runtime.create_container(&config).unwrap();

        let mut env = HashMap::new();
        env.insert("CT_VAR".into(), "ct_value".into());
        let result = container
            .exec(
                &["sh".into(), "-c".into(), "echo -n $CT_VAR".into()],
                None,
                &env,
            )
            .unwrap();

        assert_eq!(result.stdout, "ct_value");
        assert_eq!(result.exit_code, 0);
        container.remove().unwrap();
    }

    #[test]
    fn copy_to_then_copy_from_roundtrip() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-podman-ct-roundtrip");
        let _ = runtime.remove_container("ephemeral-act-test-podman-ct-roundtrip");
        let container = runtime.create_container(&config).unwrap();

        let original = b"container roundtrip data";
        let entries = vec![FileEntry {
            path: "ct_roundtrip.bin".into(),
            content: original.to_vec(),
            mode: 0o644,
        }];
        container.copy_to("/tmp", &entries).unwrap();

        let retrieved = container.copy_from("/tmp/ct_roundtrip.bin").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].path, "ct_roundtrip.bin");
        assert_eq!(retrieved[0].content, original);
        container.remove().unwrap();
    }

    #[test]
    fn get_runner_context_returns_expected_paths() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-podman-ct-context");
        let _ = runtime.remove_container("ephemeral-act-test-podman-ct-context");
        let container = runtime.create_container(&config).unwrap();

        let ctx = container.get_runner_context().unwrap();

        assert_eq!(ctx.workspace, "/workspace");
        assert_eq!(ctx.home, "/home");
        container.remove().unwrap();
    }

    #[test]
    fn remove_cleans_up_container() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-podman-ct-remove");
        let _ = runtime.remove_container("ephemeral-act-test-podman-ct-remove");
        let container = runtime.create_container(&config).unwrap();

        container.remove().unwrap();

        let _ = container.remove();
    }
}
