use std::process::Command;

use crate::core::{
    ActRunConfig, Repository, ports::outbound::ActExecutor, shared_types::ExecutionResult,
};

/// Executes GitHub Actions workflows via the `act` CLI.
pub struct GitHubActWrapper;

impl ActExecutor for GitHubActWrapper {
    fn execute_act(
        &self,
        config: &ActRunConfig,
        repository: &Repository,
    ) -> Result<ExecutionResult, String> {
        let args = Self::build_args(config, repository);
        let output = Command::new("act")
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

impl GitHubActWrapper {
    /// Builds `act` CLI arguments for GitHub repos.
    ///
    /// Extra args are passed directly to `act` without a `--` separator.
    pub fn build_args(config: &ActRunConfig, repository: &Repository) -> Vec<String> {
        let mut args = vec![
            "-C".to_string(),
            repository.path().as_path().to_string_lossy().into_owned(),
        ];

        if let Some(workflow) = config.workflow() {
            args.push("-W".to_string());
            args.push(workflow.as_str().to_string());
        }

        if let Some(job) = config.job() {
            args.push("-j".to_string());
            args.push(job.as_str().to_string());
        }

        for input in config.inputs() {
            args.push("--input".to_string());
            args.push(format!("{}={}", input.key(), input.value()));
        }

        for secret in config.secrets() {
            args.push("-s".to_string());
            args.push(secret.as_str().to_string());
        }

        args.push("--rm".to_string());
        args.push("--bind".to_string());

        if let Some(event) = config.event() {
            args.push(event.as_str().to_string());
        }

        for extra_arg in config.extra_args() {
            args.push(extra_arg.as_str().to_string());
        }

        args
    }
}
