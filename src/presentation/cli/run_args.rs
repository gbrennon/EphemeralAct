use crate::core::value_objects::{
    ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, RepoPath, RepositoryName, Secret,
};
use crate::core::{ActRunConfig, Repository};
use clap::Args;
use std::path::PathBuf;

/// CLI arguments for the `run` subcommand.
///
/// Parses all user-supplied options (workflow, job, event, inputs, secrets,
/// container engine, etc.) and maps them into the domain model via
/// [`to_domain`](Self::to_domain).
#[derive(Args)]
pub struct RunArgs {
    /// Path to the repository (defaults to the current directory).
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Container engine to use: `podman` (default) or `docker`.
    #[arg(long, default_value = "podman")]
    container_engine: String,

    /// Path to the workflow file to execute (e.g. `ci.yml`).
    #[arg(long)]
    workflow: Option<String>,

    /// Specific job name to run from the workflow.
    #[arg(long)]
    job: Option<String>,

    /// Event name that triggers the workflow (e.g. `push`, `pull_request`).
    #[arg(long)]
    event: Option<String>,

    /// Workflow inputs in `KEY=VALUE` format (repeatable).
    #[arg(long = "input", value_name = "KEY=VALUE")]
    inputs: Vec<String>,

    /// Secrets to inject into the workflow (repeatable).
    #[arg(long = "secret")]
    secrets: Vec<String>,

    /// Extra arguments forwarded directly to `act-ephemeral.sh` (repeatable).
    #[arg(long = "extra-arg")]
    extra_args: Vec<String>,

    /// Preserve the ephemeral repository after execution instead of cleaning
    /// it up.
    #[arg(long)]
    preserve: bool,
}

impl RunArgs {
    /// Converts CLI arguments into the domain model: an [`ActRunConfig`] and a
    /// [`Repository`].
    ///
    /// # Errors
    ///
    /// Returns an error if the container engine string is unrecognised or the
    /// repository path is not a valid git repository.
    pub fn to_domain(&self) -> Result<(ActRunConfig, Repository), Box<dyn std::error::Error>> {
        let container_engine = self.container_engine.parse()?;
        let repo_path = RepoPath::new(self.path.clone())?;
        let repo_name = RepositoryName::from_repo_path(&repo_path)?;
        let repository = Repository::new(repo_path, repo_name);

        let mut config = ActRunConfig::new(container_engine);

        if let Some(ref wf) = self.workflow {
            config = config.with_workflow(ActWorkflow::new(wf.clone()));
        }
        if let Some(ref job) = self.job {
            config = config.with_job(ActJob::new(job.clone()));
        }
        if let Some(ref event) = self.event {
            config = config.with_event(ActEvent::new(event.clone()));
        }
        for input_str in &self.inputs {
            let (k, v) = Self::parse_key_value(input_str)?;
            config = config.add_input(ActInput::new(k, v));
        }
        for secret_str in &self.secrets {
            config = config.add_secret(Secret::new(secret_str.clone()));
        }
        for arg_str in &self.extra_args {
            config = config.add_extra_arg(ActExtraArg::new(arg_str.clone()));
        }

        Ok((config, repository))
    }

    /// Splits a `KEY=VALUE` string into its key and value components.
    ///
    /// Returns an error string if the input doesn't contain `=`.
    fn parse_key_value(s: &str) -> Result<(String, String), String> {
        s.split_once('=')
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .ok_or_else(|| format!("expected KEY=VALUE, got '{}'", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_run_args(args: &[&str]) -> RunArgs {
        let mut full: Vec<&str> = vec!["ephemeral-act", "run"];
        full.extend_from_slice(args);
        let cli = crate::presentation::cli::CliParser::parse_from(&full);
        match cli.command {
            crate::presentation::cli::Command::Run(args) => *args,
        }
    }

    #[test]
    fn parse_key_value_with_equals() {
        let (k, v) = RunArgs::parse_key_value("KEY=value").unwrap();
        assert_eq!(k, "KEY");
        assert_eq!(v, "value");
    }

    #[test]
    fn parse_key_value_missing_equals() {
        let err = RunArgs::parse_key_value("no_equals").unwrap_err();
        assert!(err.contains("KEY=VALUE"));
    }

    #[test]
    fn to_domain_defaults() {
        let args = parse_run_args(&[]);
        let (_config, repo) = args.to_domain().unwrap();
        assert!(!repo.name().as_str().is_empty());
    }

    #[test]
    fn to_domain_with_workflow() {
        let args = parse_run_args(&["--workflow", "ci.yml"]);
        let (config, _repo) = args.to_domain().unwrap();
        let wf = config.workflow().expect("workflow should be set");
        assert_eq!(wf.as_str(), "ci.yml");
    }

    #[test]
    fn to_domain_with_job() {
        let args = parse_run_args(&["--job", "test"]);
        let (config, _repo) = args.to_domain().unwrap();
        let job = config.job().expect("job should be set");
        assert_eq!(job.as_str(), "test");
    }

    #[test]
    fn to_domain_with_event() {
        let args = parse_run_args(&["--event", "push"]);
        let (config, _repo) = args.to_domain().unwrap();
        let event = config.event().expect("event should be set");
        assert_eq!(event.as_str(), "push");
    }

    #[test]
    fn to_domain_with_inputs() {
        let args = parse_run_args(&["--input", "VAR1=val1", "--input", "VAR2=val2"]);
        let (config, _repo) = args.to_domain().unwrap();
        let inputs = config.inputs();
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].key(), "VAR1");
        assert_eq!(inputs[0].value(), "val1");
        assert_eq!(inputs[1].key(), "VAR2");
        assert_eq!(inputs[1].value(), "val2");
    }

    #[test]
    fn to_domain_with_secrets() {
        let args = parse_run_args(&["--secret", "TOKEN=abc123", "--secret", "PASS=xyz"]);
        let (config, _repo) = args.to_domain().unwrap();
        let secrets = config.secrets();
        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets[0].as_str(), "TOKEN=abc123");
        assert_eq!(secrets[1].as_str(), "PASS=xyz");
    }

    #[test]
    fn to_domain_with_extra_args() {
        let args = parse_run_args(&["--extra-arg", "verbose", "--extra-arg", "dryrun"]);
        let (config, _repo) = args.to_domain().unwrap();
        let extra = config.extra_args();
        assert_eq!(extra.len(), 2);
        assert_eq!(extra[0].as_str(), "verbose");
        assert_eq!(extra[1].as_str(), "dryrun");
    }

    #[test]
    fn to_domain_container_engine_docker() {
        let args = parse_run_args(&["--container-engine", "docker"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(matches!(
            config.container_engine(),
            &crate::core::value_objects::ContainerEngine::Docker
        ));
    }

    #[test]
    fn to_domain_invalid_container_engine() {
        let args = parse_run_args(&["--container-engine", "invalid"]);
        let err = args.to_domain().unwrap_err();
        assert!(err.to_string().contains("invalid"));
    }
}
