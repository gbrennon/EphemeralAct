use std::path::PathBuf;

use clap::Args;

use crate::core::{
    ActRunConfig, Repository,
    value_objects::{ActEvent, ActInput, ActJob, ActWorkflow, RepoPath, RepositoryName, Secret},
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

    /// Secrets in `KEY=VALUE` format, or `KEY` alone to read the value from
    /// the environment (repeatable).
    #[arg(long = "secret", value_name = "KEY[=VALUE]")]
    secrets: Vec<String>,

    /// Run every workflow found in the repository instead of a single one.
    #[arg(long = "all-workflows")]
    all_workflows: bool,

    /// Preserve the ephemeral repository after execution instead of cleaning
    /// it up.
    #[arg(long)]
    preserve: bool,

    /// Use the real Docker or Podman adapter instead of the default runtime
    /// selection.
    #[arg(long = "allow-real-container")]
    allow_real_container: bool,

    /// Use the real action fetcher that contacts the forge instead of a local
    /// mirror.
    #[arg(long = "allow-real-fetcher")]
    allow_real_fetcher: bool,

    /// Allow containers to make outbound network requests.
    #[arg(long = "allow-network")]
    allow_network: bool,
}

impl RunArgs {
    /// Converts CLI arguments into the domain model: an [`ActRunConfig`] and a
    /// [`Repository`].
    ///
    /// # Errors
    ///
    /// Returns an error if the repository path is not a valid git repository.
    pub fn to_domain(&self) -> Result<(ActRunConfig, Repository), Box<dyn std::error::Error>> {
        let repo_path = RepoPath::new(self.path.clone()).map_err(|e| format!("{:?}", e))?;
        let repo_name =
            RepositoryName::from_repo_path(&repo_path).map_err(|e| format!("{:?}", e))?;
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
        config = config.with_all_workflows(self.all_workflows);
        config = config.with_allow_real_container(self.allow_real_container);
        config = config.with_allow_real_fetcher(self.allow_real_fetcher);
        config = config.with_allow_network(self.allow_network);
        for input_str in &self.inputs {
            let (k, v) = Self::parse_key_value(input_str)?;
            config = config.add_input(ActInput::new(k, v));
        }
        for secret_str in &self.secrets {
            let (name, value) = Self::parse_secret(secret_str)?;
            config = config.add_secret(Secret::new(name, value));
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

    /// Splits a secret argument into its name and value.
    ///
    /// `KEY=VALUE` supplies the value inline; a bare `KEY` reads it from the
    /// environment variable of the same name.
    pub fn parse_secret(s: &str) -> Result<(String, String), String> {
        match s.split_once('=') {
            Some((name, value)) => Ok((name.to_string(), value.to_string())),
            None => std::env::var(s)
                .map(|value| (s.to_string(), value))
                .map_err(|_| {
                    format!("secret '{s}' has no value and no environment variable is set")
                }),
        }
    }
}
