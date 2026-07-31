use crate::core::ports::inbound::run_act_port::RunActUseCase;
use crate::core::ports::outbound::ActExecutor;
use crate::core::shared_types::ExecutionResult;
use crate::core::{ActRunConfig, Repository};

pub struct RunActService<A: ActExecutor> {
    executor: A,
}

impl<A: ActExecutor> RunActService<A> {
    pub fn new(executor: A) -> Self {
        Self { executor }
    }

    fn build_args(config: &ActRunConfig, repository: &Repository) -> Vec<String> {
        let mut args = Vec::new();

        // Repository path as positional arg
        args.push(repository.path().as_path().to_string_lossy().into_owned());

        // Container engine
        args.push("--container-engine".to_string());
        match config.container_engine() {
            crate::core::value_objects::ContainerEngine::Podman => args.push("podman".to_string()),
            crate::core::value_objects::ContainerEngine::Docker => args.push("docker".to_string()),
        }

        // Container daemon socket
        args.push("--container-daemon-socket".to_string());
        args.push(config.container_daemon_socket().as_str().to_string());

        // Workflow (if set)
        if let Some(workflow) = config.workflow() {
            args.push("--workflow".to_string());
            args.push(workflow.as_str().to_string());
        }

        // Job (if set)
        if let Some(job) = config.job() {
            args.push("--job".to_string());
            args.push(job.as_str().to_string());
        }

        // Event (if set)
        if let Some(event) = config.event() {
            args.push("--event".to_string());
            args.push(event.as_str().to_string());
        }

        // Inputs (repeatable)
        for input in config.inputs() {
            args.push("--input".to_string());
            args.push(format!("{}={}", input.key(), input.value()));
        }

        // Secrets (repeatable)
        for secret in config.secrets() {
            args.push("--secret".to_string());
            args.push(secret.as_str().to_string());
        }

        // Extra args (repeatable)
        for extra_arg in config.extra_args() {
            args.push("--extra-arg".to_string());
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

impl<A: ActExecutor> RunActUseCase for RunActService<A> {
    fn run_act(
        &self,
        config: ActRunConfig,
        repository: Repository,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        let args = Self::build_args(&config, &repository);
        self.executor.execute(&args).map_err(|e| e.into())
    }
}
