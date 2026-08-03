use ephemeral_act::core::ports::inbound::run_act_port::RunActUseCase;
use ephemeral_act::core::ports::outbound::ActExecutor;
use ephemeral_act::core::services::run_act_service::RunActService;
use ephemeral_act::core::shared_types::ExecutionResult;
use ephemeral_act::core::value_objects::{
    ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, ContainerDaemonSocket, ContainerEngine,
    RepoPath, RepositoryName, Secret,
};
use ephemeral_act::core::{ActRunConfig, Repository};
use std::cell::RefCell;
use std::rc::Rc;

/// Lightweight fake that records received config + repository and returns a
/// pre-configured result.
struct FakeActExecutor {
    captured_config: Rc<RefCell<Option<ActRunConfig>>>,
    captured_repo: Rc<RefCell<Option<Repository>>>,
    result: Rc<Result<ExecutionResult, String>>,
}

impl ActExecutor for FakeActExecutor {
    fn execute_act(
        &self,
        config: &ActRunConfig,
        repository: &Repository,
    ) -> Result<ExecutionResult, String> {
        *self.captured_config.borrow_mut() = Some(config.clone());
        *self.captured_repo.borrow_mut() = Some(repository.clone());
        self.result.as_ref().clone()
    }
}

fn fake_executor(
    result: Result<ExecutionResult, String>,
) -> (
    FakeActExecutor,
    Rc<RefCell<Option<ActRunConfig>>>,
    Rc<RefCell<Option<Repository>>>,
) {
    let captured_config = Rc::new(RefCell::new(None));
    let captured_repo = Rc::new(RefCell::new(None));
    (
        FakeActExecutor {
            captured_config: Rc::clone(&captured_config),
            captured_repo: Rc::clone(&captured_repo),
            result: Rc::new(result),
        },
        captured_config,
        captured_repo,
    )
}

fn ok_result() -> ExecutionResult {
    ExecutionResult {
        success: true,
        stdout: "mock stdout".into(),
        stderr: String::new(),
    }
}

fn test_repository() -> Repository {
    Repository::new(
        RepoPath::new(env!("CARGO_MANIFEST_DIR").to_string()).unwrap(),
        RepositoryName::new("test-repo".into()).unwrap(),
    )
}

fn minimal_config() -> ActRunConfig {
    ActRunConfig::new(
        ContainerEngine::Podman,
        ContainerDaemonSocket::new("unix:///run/podman/podman.sock".into()),
    )
}

#[test]
fn passes_repository_to_executor() {
    let (fake, _, captured_repo) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let repo = test_repository();

    service.run_act(minimal_config(), repo.clone()).unwrap();

    let captured = captured_repo.borrow();
    assert_eq!(captured.as_ref().unwrap().path(), repo.path());
    assert_eq!(captured.as_ref().unwrap().name(), repo.name());
}

#[test]
fn passes_container_daemon_socket_config() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);

    service
        .run_act(minimal_config(), test_repository())
        .unwrap();

    let config = captured_config.borrow();
    assert_eq!(
        config.as_ref().unwrap().container_daemon_socket().as_str(),
        "unix:///run/podman/podman.sock"
    );
}

#[test]
fn passes_default_rm_and_bind_flags() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);

    service
        .run_act(minimal_config(), test_repository())
        .unwrap();

    let config = captured_config.borrow();
    assert!(config.as_ref().unwrap().rm());
    assert!(config.as_ref().unwrap().bind());
}

#[test]
fn passes_rm_false_when_disabled() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_rm(false);

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    assert!(!captured.as_ref().unwrap().rm());
}

#[test]
fn passes_bind_false_when_disabled() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_bind(false);

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    assert!(!captured.as_ref().unwrap().bind());
}

#[test]
fn passes_workflow_when_set() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_workflow(ActWorkflow::new("ci.yaml".into()));

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    assert_eq!(
        captured.as_ref().unwrap().workflow().unwrap().as_str(),
        "ci.yaml"
    );
}

#[test]
fn passes_job_when_set() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_job(ActJob::new("build".into()));

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    assert_eq!(captured.as_ref().unwrap().job().unwrap().as_str(), "build");
}

#[test]
fn passes_event_when_set() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_event(ActEvent::new("push".into()));

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    assert_eq!(captured.as_ref().unwrap().event().unwrap().as_str(), "push");
}

#[test]
fn passes_inputs() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config()
        .add_input(ActInput::new("debug".into(), "true".into()))
        .add_input(ActInput::new("target".into(), "x86_64".into()));

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    let inputs = captured.as_ref().unwrap().inputs();
    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[0].key(), "debug");
    assert_eq!(inputs[0].value(), "true");
    assert_eq!(inputs[1].key(), "target");
    assert_eq!(inputs[1].value(), "x86_64");
}

#[test]
fn passes_secrets() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().add_secret(Secret::new("GITHUB_TOKEN".into()));

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    let secrets = captured.as_ref().unwrap().secrets();
    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].as_str(), "GITHUB_TOKEN");
}

#[test]
fn passes_extra_args() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config()
        .add_extra_arg(ActExtraArg::new("--verbose".into()))
        .add_extra_arg(ActExtraArg::new("--dryrun".into()));

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    let extra_args = captured.as_ref().unwrap().extra_args();
    assert_eq!(extra_args.len(), 2);
    assert_eq!(extra_args[0].as_str(), "--verbose");
    assert_eq!(extra_args[1].as_str(), "--dryrun");
}

#[test]
fn passes_docker_daemon_socket() {
    let (fake, captured_config, _) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = ActRunConfig::new(
        ContainerEngine::Docker,
        ContainerDaemonSocket::new("unix:///var/run/docker.sock".into()),
    );

    service.run_act(config, test_repository()).unwrap();

    let captured = captured_config.borrow();
    assert_eq!(
        captured
            .as_ref()
            .unwrap()
            .container_daemon_socket()
            .as_str(),
        "unix:///var/run/docker.sock"
    );
}

#[test]
fn propagates_executor_error() {
    let error_msg = "act: command not found".to_string();
    let (fake, _, _) = fake_executor(Err(error_msg.clone()));
    let service = RunActService::new(fake);

    let result = service.run_act(minimal_config(), test_repository());

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains(&error_msg));
}

#[test]
fn returns_executor_success_result() {
    let expected = ExecutionResult {
        success: true,
        stdout: "job output\n".into(),
        stderr: "some warning\n".into(),
    };
    let result_clone = ExecutionResult {
        success: expected.success,
        stdout: expected.stdout.clone(),
        stderr: expected.stderr.clone(),
    };
    let (fake, _, _) = fake_executor(Ok(result_clone));
    let service = RunActService::new(fake);

    let result = service
        .run_act(minimal_config(), test_repository())
        .unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, expected.stdout);
    assert_eq!(result.stderr, expected.stderr);
}
