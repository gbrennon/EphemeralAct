use std::path::PathBuf;

use clap::Args;

use crate::core::{
    ActRunConfig, Repository,
    value_objects::{
        ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, RepoPath, RepositoryName, Secret,
    },
};

/// CLI arguments for the `run` subcommand.
///
/// Parses all user-supplied options (workflow, job, event, inputs, secrets,
/// etc.) and maps them into the domain model via
/// [`to_domain`](Self::to_domain). The container runtime is auto-detected
/// (Docker or Podman) at execution time.
#[derive(Args)]
pub struct RunArgs {
    /// Path to the repository (defaults to the current directory).
    #[arg(default_value = ".")]
    path: PathBuf,
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

    /// Extra arguments forwarded directly to `act` (repeatable).
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
    /// Returns an error if the repository path is not a valid git repository.
    pub fn to_domain(&self) -> Result<(ActRunConfig, Repository), Box<dyn std::error::Error>> {
        let repo_path = RepoPath::new(self.path.clone())?;
        let repo_name = RepositoryName::from_repo_path(&repo_path)?;
        let repository = Repository::new(repo_path, repo_name);

        let mut config = ActRunConfig::new();

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
    pub fn parse_key_value(s: &str) -> Result<(String, String), String> {
        s.split_once('=')
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .ok_or_else(|| format!("expected KEY=VALUE, got '{}'", s))
    }
}
