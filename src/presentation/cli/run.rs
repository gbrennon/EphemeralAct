use crate::core::ports::inbound::run_act_port::RunActUseCase;
use crate::core::value_objects::{
    ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, ContainerDaemonSocket, RepoPath,
    RepositoryName, Secret,
};
use crate::core::{ActRunConfig, Repository};
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct RunArgs {
    /// Path to the repository
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Container engine (podman or docker)
    #[arg(long, default_value = "podman")]
    container_engine: String,

    /// Container daemon socket URI
    #[arg(long, default_value = "unix:///run/podman/podman.sock")]
    container_daemon_socket: String,

    /// Workflow file path
    #[arg(long)]
    workflow: Option<String>,

    /// Job name
    #[arg(long)]
    job: Option<String>,

    /// Event name
    #[arg(long)]
    event: Option<String>,

    /// Inputs in key=value format (repeatable)
    #[arg(long = "input", value_name = "KEY=VALUE")]
    inputs: Vec<String>,

    /// Secrets (repeatable)
    #[arg(long = "secret")]
    secrets: Vec<String>,

    /// Extra arguments passed through to act (repeatable)
    #[arg(long = "extra-arg")]
    extra_args: Vec<String>,

    /// Remove container after execution (default: true)
    #[arg(long, default_value = "true")]
    rm: bool,

    /// Bind mount working directory (default: true)
    #[arg(long, default_value = "true")]
    bind: bool,

    /// Preserve the ephemeral repository after execution
    #[arg(long)]
    preserve: bool,
}

impl RunArgs {
    pub fn to_domain(&self) -> Result<(ActRunConfig, Repository), Box<dyn std::error::Error>> {
        let container_engine = self.container_engine.parse()?;
        let daemon_socket = ContainerDaemonSocket::new(self.container_daemon_socket.clone());
        let repo_path = RepoPath::new(self.path.clone())?;
        let repo_name = RepositoryName::from_repo_path(&repo_path)?;
        let repository = Repository::new(repo_path, repo_name);

        let mut config = ActRunConfig::new(container_engine, daemon_socket);

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
            let (k, v) = parse_key_value(input_str)?;
            config = config.add_input(ActInput::new(k, v));
        }
        for secret_str in &self.secrets {
            config = config.add_secret(Secret::new(secret_str.clone()));
        }
        for arg_str in &self.extra_args {
            config = config.add_extra_arg(ActExtraArg::new(arg_str.clone()));
        }
        if !self.rm {
            config = config.with_rm(false);
        }
        if !self.bind {
            config = config.with_bind(false);
        }

        Ok((config, repository))
    }
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| format!("expected KEY=VALUE, got '{}'", s))
}

pub fn execute<U: RunActUseCase>(
    args: RunArgs,
    use_case: U,
) -> Result<(), Box<dyn std::error::Error>> {
    let (config, repository) = args.to_domain()?;
    let result = use_case.run_act(config, repository)?;

    if !result.stdout.is_empty() {
        println!("{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        eprintln!("{}", result.stderr);
    }
    if !result.success {
        std::process::exit(1);
    }
    Ok(())
}
