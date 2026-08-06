use std::process::Command;

use crate::core::{
    ActRunConfig, Repository, ports::outbound::ActExecutor, shared_types::ExecutionResult,
};

/// Executes Forgejo Actions workflows via the `act` CLI.
pub struct ForgejoActWrapper;

impl ActExecutor for ForgejoActWrapper {
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

impl ForgejoActWrapper {
    /// Builds `act` CLI arguments for Forgejo repos.
    ///
    /// Scans `.forgejo/workflows/` for `runs-on` labels and automatically
    /// injects `-P` platform mappings so `act` can resolve Forgejo runner
    /// labels to Docker images. Passes `--workflows .forgejo/workflows/`
    /// directly to `act` so it can discover Forgejo workflow files.
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

        args.push("--workflows".to_string());
        args.push(".forgejo/workflows/".to_string());

        let platform_args = Self::forgejo_platform_mappings(repository);
        args.extend(platform_args);

        if let Some(event) = config.event() {
            args.push(event.as_str().to_string());
        }

        for extra_arg in config.extra_args() {
            args.push(extra_arg.as_str().to_string());
        }

        args
    }

    /// Scans `.forgejo/workflows/` for `runs-on` labels and returns `-P`
    /// platform-mapping arguments that tell `act` which Docker image to use
    /// for each Forgejo runner label.
    ///
    /// # Mapping
    ///
    /// | Label               | Docker Image                          |
    /// |---------------------|---------------------------------------|
    /// | `codeberg-tiny`     | `catthehacker/ubuntu:act-latest`      |
    /// | `codeberg-medium`   | `catthehacker/ubuntu:act-latest`      |
    /// | `ubuntu-latest`     | `catthehacker/ubuntu:act-latest`      |
    /// | `ubuntu-22.04`      | `catthehacker/ubuntu:act-22.04`       |
    /// | (any other)         | `catthehacker/ubuntu:act-latest`      |
    fn forgejo_platform_mappings(repository: &Repository) -> Vec<String> {
        let workflows_dir = repository
            .path()
            .as_path()
            .join(".forgejo")
            .join("workflows");

        let mut labels = std::collections::BTreeSet::new();

        if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|ext| ext == "yml" || ext == "yaml")
                    && let Ok(content) = std::fs::read_to_string(&path)
                {
                    for line in content.lines() {
                        let trimmed = line.trim_start();
                        if let Some(label) = trimmed
                            .strip_prefix("runs-on:")
                            .or_else(|| trimmed.strip_prefix("runs-on: "))
                        {
                            let label = label.trim().trim_matches('"').trim_matches('\'');
                            if !label.is_empty() {
                                labels.insert(label.to_string());
                            }
                        }
                    }
                }
            }
        }

        let mut args = Vec::new();
        for label in &labels {
            let image = Self::resolve_platform_image(label);
            args.push("-P".to_string());
            args.push(format!("{}={}", label, image));
        }
        args
    }

    /// Maps a Forgejo runner label to a Docker image suitable for `act`.
    fn resolve_platform_image(label: &str) -> &'static str {
        match label {
            "ubuntu-22.04" => "catthehacker/ubuntu:act-22.04",
            _ => "catthehacker/ubuntu:act-latest",
        }
    }
}
