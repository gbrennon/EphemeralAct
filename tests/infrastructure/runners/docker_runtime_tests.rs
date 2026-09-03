use std::collections::HashMap;

use ephact::{
    application::ports::outbound::{ContainerConfig, ContainerRuntimePort, FileEntry},
    infrastructure::runners::DockerRuntime,
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
            match DockerRuntime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("SKIP: Docker runtime not available: {e:?}");
                    return;
                }
            }
        };
    }

    #[test]
    fn new_connects_to_socket() {
        let _runtime = runtime!();
    }

    #[test]
    fn get_host_info_returns_valid_data() {
        let runtime = runtime!();
        let info = runtime.get_host_info().unwrap();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn pull_nonexistent_image_fails() {
        let runtime = runtime!();
        assert!(
            runtime
                .pull_image("nonexistent-image-xyz-123:latest", None)
                .is_err()
        );
    }

    #[test]
    fn create_and_remove_container_lifecycle() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-lifecycle");
        let _ = runtime.remove_container("ephemeral-act-test-docker-lifecycle");
        let container = runtime.create_container(&config).unwrap();
        container.remove().unwrap();
    }

    #[test]
    fn stop_nonexistent_container_is_noop() {
        let runtime = runtime!();
        let _ = runtime.stop_container("nonexistent-container-xyz-123");
    }

    #[test]
    fn remove_nonexistent_container_is_noop() {
        let runtime = runtime!();
        let _ = runtime.remove_container("nonexistent-container-xyz-123");
    }

    #[test]
    fn exec_echo_returns_stdout() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-exec");
        let _ = runtime.remove_container("ephemeral-act-test-docker-exec");
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
    fn pull_image_with_platform_succeeds() {
        let runtime = runtime!();
        let result = runtime.pull_image("alpine:latest", Some("linux/amd64"));
        assert!(result.is_ok());
    }

    #[test]
    fn stop_running_container_succeeds() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-stop");
        let _ = runtime.remove_container("ephemeral-act-test-docker-stop");
        let container = runtime.create_container(&config).unwrap();
        runtime
            .stop_container("ephemeral-act-test-docker-stop")
            .unwrap();
        container.remove().unwrap();
    }

    #[test]
    fn exec_with_workdir_runs_in_specified_directory() {
        let runtime = runtime!();
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
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-env");
        let _ = runtime.remove_container("ephemeral-act-test-docker-env");
        let container = runtime.create_container(&config).unwrap();
        let mut env = HashMap::new();
        env.insert("MY_VAR".into(), "my_value".into());
        let result = container
            .exec(
                &["sh".into(), "-c".into(), "echo -n $MY_VAR".into()],
                None,
                &env,
            )
            .unwrap();
        assert_eq!(result.stdout, "my_value");
        assert_eq!(result.exit_code, 0);
        container.remove().unwrap();
    }

    #[test]
    fn get_runner_context_returns_expected_paths() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-context");
        let _ = runtime.remove_container("ephemeral-act-test-docker-context");
        let container = runtime.create_container(&config).unwrap();
        let ctx = container.get_runner_context().unwrap();
        assert_eq!(ctx.workspace, "/workspace");
        assert_eq!(ctx.home, "/home");
        container.remove().unwrap();
    }

    #[test]
    fn copy_to_creates_file_in_container() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-copyto");
        let _ = runtime.remove_container("ephemeral-act-test-docker-copyto");
        let container = runtime.create_container(&config).unwrap();

        let entries = vec![FileEntry {
            path: "test.txt".into(),
            content: b"hello copy_to".to_vec(),
            mode: 0o644,
        }];
        container.copy_to("/tmp", &entries).unwrap();

        let result = container
            .exec(
                &["cat".into(), "/tmp/test.txt".into()],
                None,
                &HashMap::new(),
            )
            .unwrap();
        assert_eq!(result.stdout, "hello copy_to");
        assert_eq!(result.exit_code, 0);
        container.remove().unwrap();
    }

    #[test]
    fn copy_from_reads_file_from_container() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-copyfrom");
        let _ = runtime.remove_container("ephemeral-act-test-docker-copyfrom");
        let container = runtime.create_container(&config).unwrap();

        container
            .exec(
                &[
                    "sh".into(),
                    "-c".into(),
                    "echo -n 'hello copy_from' > /tmp/from.txt".into(),
                ],
                None,
                &HashMap::new(),
            )
            .unwrap();

        let entries = container.copy_from("/tmp/from.txt").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "from.txt");
        assert_eq!(entries[0].content, b"hello copy_from");
        container.remove().unwrap();
    }

    #[test]
    fn copy_to_and_copy_from_roundtrip() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-roundtrip");
        let _ = runtime.remove_container("ephemeral-act-test-docker-roundtrip");
        let container = runtime.create_container(&config).unwrap();

        let original = b"roundtrip data 12345";
        let entries = vec![FileEntry {
            path: "roundtrip.bin".into(),
            content: original.to_vec(),
            mode: 0o644,
        }];
        container.copy_to("/tmp", &entries).unwrap();

        let retrieved = container.copy_from("/tmp/roundtrip.bin").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].content, original);
        container.remove().unwrap();
    }

    #[test]
    fn exec_failing_command_returns_nonzero_exit() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-exitcode");
        let _ = runtime.remove_container("ephemeral-act-test-docker-exitcode");
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
    fn remove_with_force_cleans_up_container() {
        let runtime = runtime!();
        let config = make_config("ephemeral-act-test-docker-removeforce");
        let _ = runtime.remove_container("ephemeral-act-test-docker-removeforce");
        let container = runtime.create_container(&config).unwrap();

        container.remove().unwrap();

        let _ = container.remove();
    }
}
