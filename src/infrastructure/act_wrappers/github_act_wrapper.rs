use crate::core::ports::outbound::ActExecutor;
use crate::core::shared_types::ExecutionResult;
use crate::core::{ActRunConfig, Repository};
use std::process::Command;

/// Executes GitHub Actions workflows via the `act-ephemeral.sh` wrapper.
pub struct GitHubActWrapper;

impl ActExecutor for GitHubActWrapper {
    fn execute_act(
        &self,
        config: &ActRunConfig,
        repository: &Repository,
    ) -> Result<ExecutionResult, String> {
        let args = Self::build_args(config, repository);
        let output = Command::new("act-ephemeral.sh")
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
    /// Builds `act-ephemeral.sh` CLI arguments for GitHub repos.
    ///
    /// Extra args are passed after `--` to be forwarded to the underlying
    /// `act` command.
    pub fn build_args(config: &ActRunConfig, repository: &Repository) -> Vec<String> {
        let mut args = vec![
            repository.path().as_path().to_string_lossy().into_owned(),
            "-c".to_string(),
            config.container_engine().as_str().to_string(),
        ];

        if let Some(workflow) = config.workflow() {
            args.push("-w".to_string());
            args.push(workflow.as_str().to_string());
        }

        if let Some(job) = config.job() {
            args.push("-j".to_string());
            args.push(job.as_str().to_string());
        }

        if let Some(event) = config.event() {
            args.push("-e".to_string());
            args.push(event.as_str().to_string());
        }

        for input in config.inputs() {
            args.push("-i".to_string());
            args.push(format!("{}={}", input.key(), input.value()));
        }

        for secret in config.secrets() {
            args.push("-s".to_string());
            args.push(secret.as_str().to_string());
        }

        let has_extras = !config.extra_args().is_empty();
        if has_extras {
            args.push("--".to_string());
            for extra_arg in config.extra_args() {
                args.push(extra_arg.as_str().to_string());
            }
        }

        args
    }
}
