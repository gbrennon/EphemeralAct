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
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    rm: bool,

    /// Bind mount working directory (default: true)
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
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
            let (k, v) = Self::parse_key_value(input_str)?;
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

    fn parse_key_value(s: &str) -> Result<(String, String), String> {
        s.split_once('=')
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .ok_or_else(|| format!("expected KEY=VALUE, got '{}'", s))
    }

    pub fn execute<U: RunActUseCase>(self, use_case: U) -> Result<(), Box<dyn std::error::Error>> {
        let (config, repository) = self.to_domain()?;
        let result = use_case.run_act(config, repository)?;

        if !result.stdout.is_empty() {
            println!("{}", result.stdout);
        }
        if !result.stderr.is_empty() {
            eprintln!("{}", result.stderr);
        }
        if !result.success {
            return Err("workflow failed".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ports::inbound::run_act_port::RunActUseCase;
    use crate::core::shared_types::ExecutionResult;
    use clap::Parser;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    struct StubUseCase {
        result: Result<ExecutionResult, String>,
    }

    impl RunActUseCase for StubUseCase {
        fn run_act(
            &self,
            _config: ActRunConfig,
            _repository: Repository,
        ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
            self.result
                .clone()
                .map_err(|e| Box::<dyn std::error::Error>::from(e))
        }
    }

    fn parse_run_args(args: &[&str]) -> RunArgs {
        let mut full: Vec<&str> = vec!["ephemeral-act", "run"];
        full.extend_from_slice(args);
        let cli = crate::presentation::cli::Cli::parse_from(&full);
        match cli.command {
            crate::presentation::cli::Command::Run(args) => args,
        }
    }

    // -----------------------------------------------------------------------
    // parse_key_value
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // to_domain
    // -----------------------------------------------------------------------

    #[test]
    fn to_domain_defaults() {
        let args = parse_run_args(&[]);
        let (config, repo) = args.to_domain().unwrap();
        assert!(!repo.name().as_str().is_empty());
        assert!(config.rm());
        assert!(config.bind());
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
    fn to_domain_rm_false() {
        let args = parse_run_args(&["--rm=false"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(!config.rm());
    }

    #[test]
    fn to_domain_bind_false() {
        let args = parse_run_args(&["--bind=false"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(!config.bind());
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

    #[test]
    fn to_domain_custom_socket() {
        let args = parse_run_args(&["--container-daemon-socket", "unix:///custom.sock"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert_eq!(config.container_daemon_socket().as_str(), "unix:///custom.sock");
    }

    // -----------------------------------------------------------------------
    // execute
    // -----------------------------------------------------------------------

    #[test]
    fn execute_success_with_output() {
        let args = parse_run_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: "build completed".into(),
                stderr: String::new(),
            }),
        };
        let result = args.execute(use_case);
        assert!(result.is_ok());
    }

    #[test]
    fn execute_failure_with_stderr() {
        let args = parse_run_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: "error: build failed".into(),
            }),
        };
        let err = args.execute(use_case).unwrap_err();
        assert!(err.to_string().contains("workflow failed"));
    }

    #[test]
    fn execute_use_case_error_propagates() {
        let args = parse_run_args(&[]);
        let use_case = StubUseCase {
            result: Err("something broke".into()),
        };
        let err = args.execute(use_case).unwrap_err();
        assert!(err.to_string().contains("something broke"));
    }
}
