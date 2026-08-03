use crate::core::ports::outbound::ActExecutor;
use crate::core::shared_types::ExecutionResult;
use crate::core::{ActRunConfig, Repository};
use std::process::Command;

/// Executes Forgejo Actions workflows via the `act_runner` CLI.
pub struct ForgejoActWrapper;

impl ActExecutor for ForgejoActWrapper {
    fn execute_act(
        &self,
        config: &ActRunConfig,
        repository: &Repository,
    ) -> Result<ExecutionResult, String> {
        let args = Self::build_args(config, repository);
        let output = Command::new("act_runner")
            .args(&args)
            .output()
            .map_err(|e| e.to_string())?;

        Ok(ExecutionResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

impl ForgejoActWrapper {
    /// Builds Forgejo Actions-specific CLI arguments.
    ///
    /// Uses `--workflows .forgejo/workflows/` to point at the Forgejo workflow
    /// directory since `act_runner` does not auto-detect `.forgejo/`.
    pub fn build_args(config: &ActRunConfig, repository: &Repository) -> Vec<String> {
        let mut args = vec![
            // Working directory
            "-C".to_string(),
            repository.path().as_path().to_string_lossy().into_owned(),
            // Container daemon socket
            "--container-daemon-socket".to_string(),
            config.container_daemon_socket().as_str().to_string(),
            // Point at Forgejo workflows directory
            "--workflows".to_string(),
            ".forgejo/workflows/".to_string(),
        ];

        // Workflow (if set) — overrides the directory default
        if let Some(workflow) = config.workflow() {
            args.push("-W".to_string());
            args.push(workflow.as_str().to_string());
        }

        // Job (if set)
        if let Some(job) = config.job() {
            args.push("-j".to_string());
            args.push(job.as_str().to_string());
        }

        // Event as positional arg (if set)
        if let Some(event) = config.event() {
            args.push(event.as_str().to_string());
        }

        // Inputs (repeatable)
        for input in config.inputs() {
            args.push("--input".to_string());
            args.push(format!("{}={}", input.key(), input.value()));
        }

        // Secrets (repeatable)
        for secret in config.secrets() {
            args.push("-s".to_string());
            args.push(secret.as_str().to_string());
        }

        // Extra args — pass through directly
        for extra_arg in config.extra_args() {
            args.push(extra_arg.as_str().to_string());
        }

        // Flags
        if config.rm() {
            args.push("--rm".to_string());
        }
        if config.bind() {
            args.push("--bind".to_string());
        }

        args
    }
}
