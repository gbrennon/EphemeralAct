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
use std::env;
use std::rc::Rc;

/// Lightweight fake that records received args and returns a pre-configured result.
struct FakeActExecutor {
    captured_args: Rc<RefCell<Vec<String>>>,
    result: Rc<Result<ExecutionResult, String>>,
}

impl ActExecutor for FakeActExecutor {
    fn execute(&self, args: &[String]) -> Result<ExecutionResult, String> {
        *self.captured_args.borrow_mut() = args.to_vec();
        self.result.as_ref().clone()
    }
}

fn fake_executor(
    result: Result<ExecutionResult, String>,
) -> (FakeActExecutor, Rc<RefCell<Vec<String>>>) {
    let args = Rc::new(RefCell::new(Vec::new()));
    (
        FakeActExecutor {
            captured_args: Rc::clone(&args),
            result: Rc::new(result),
        },
        args,
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
    let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = RepoPath::new(crate_root).unwrap();
    let name = RepositoryName::new("test-repo".into()).unwrap();
    Repository::new(path, name)
}

fn minimal_config() -> ActRunConfig {
    ActRunConfig::new(
        ContainerEngine::Podman,
        ContainerDaemonSocket::new("unix:///run/podman/podman.sock".into()),
    )
}

#[test]
fn passes_repo_path_as_positional_arg() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let repo = test_repository();

    service.run_act(minimal_config(), repo).unwrap();

    let args = captured.borrow();
    let expected_path = test_repository()
        .path()
        .as_path()
        .to_string_lossy()
        .into_owned();
    assert_eq!(args[0], expected_path);
}

#[test]
fn includes_container_engine_flag() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);

    service
        .run_act(minimal_config(), test_repository())
        .unwrap();

    let args = captured.borrow();
    let engine_pos = args
        .iter()
        .position(|a| a == "--container-engine")
        .expect("missing --container-engine");
    assert_eq!(args[engine_pos + 1], "podman");
}

#[test]
fn includes_container_daemon_socket_flag() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);

    service
        .run_act(minimal_config(), test_repository())
        .unwrap();

    let args = captured.borrow();
    let socket_pos = args
        .iter()
        .position(|a| a == "--container-daemon-socket")
        .expect("missing --container-daemon-socket");
    assert_eq!(args[socket_pos + 1], "unix:///run/podman/podman.sock");
}

#[test]
fn defaults_rm_and_bind_to_true() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);

    service
        .run_act(minimal_config(), test_repository())
        .unwrap();

    let args = captured.borrow();
    assert!(args.contains(&"--rm".to_string()), "expected --rm flag");
    assert!(args.contains(&"--bind".to_string()), "expected --bind flag");
}

#[test]
fn respects_rm_false() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_rm(false);

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    assert!(!args.contains(&"--rm".to_string()));
}

#[test]
fn respects_bind_false() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_bind(false);

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    assert!(!args.contains(&"--bind".to_string()));
}

#[test]
fn includes_workflow_when_set() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_workflow(ActWorkflow::new("ci.yaml".into()));

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    let wf_pos = args
        .iter()
        .position(|a| a == "--workflow")
        .expect("missing --workflow");
    assert_eq!(args[wf_pos + 1], "ci.yaml");
}

#[test]
fn includes_job_when_set() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_job(ActJob::new("build".into()));

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    let job_pos = args
        .iter()
        .position(|a| a == "--job")
        .expect("missing --job");
    assert_eq!(args[job_pos + 1], "build");
}

#[test]
fn includes_event_when_set() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().with_event(ActEvent::new("push".into()));

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    let ev_pos = args
        .iter()
        .position(|a| a == "--event")
        .expect("missing --event");
    assert_eq!(args[ev_pos + 1], "push");
}

#[test]
fn includes_inputs_as_key_equals_value() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config()
        .add_input(ActInput::new("debug".into(), "true".into()))
        .add_input(ActInput::new("target".into(), "x86_64".into()));

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    let input_indices: Vec<_> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--input")
        .collect();
    assert_eq!(input_indices.len(), 2);
    assert_eq!(args[input_indices[0].0 + 1], "debug=true");
    assert_eq!(args[input_indices[1].0 + 1], "target=x86_64");
}

#[test]
fn includes_secrets() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config().add_secret(Secret::new("GITHUB_TOKEN".into()));

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    let sec_pos = args
        .iter()
        .position(|a| a == "--secret")
        .expect("missing --secret");
    assert_eq!(args[sec_pos + 1], "GITHUB_TOKEN");
}

#[test]
fn includes_extra_args() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = minimal_config()
        .add_extra_arg(ActExtraArg::new("--verbose".into()))
        .add_extra_arg(ActExtraArg::new("--dryrun".into()));

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    let extra_indices: Vec<_> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--extra-arg")
        .collect();
    assert_eq!(extra_indices.len(), 2);
    assert_eq!(args[extra_indices[0].0 + 1], "--verbose");
    assert_eq!(args[extra_indices[1].0 + 1], "--dryrun");
}

#[test]
fn includes_docker_engine_when_configured() {
    let (fake, captured) = fake_executor(Ok(ok_result()));
    let service = RunActService::new(fake);
    let config = ActRunConfig::new(
        ContainerEngine::Docker,
        ContainerDaemonSocket::new("unix:///var/run/docker.sock".into()),
    );

    service.run_act(config, test_repository()).unwrap();

    let args = captured.borrow();
    let engine_pos = args
        .iter()
        .position(|a| a == "--container-engine")
        .expect("missing --container-engine");
    assert_eq!(args[engine_pos + 1], "docker");
}

#[test]
fn propagates_executor_error() {
    let error_msg = "act: command not found".to_string();
    let (fake, _captured) = fake_executor(Err(error_msg.clone()));
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
    let (fake, _captured) = fake_executor(Ok(result_clone));
    let service = RunActService::new(fake);

    let result = service
        .run_act(minimal_config(), test_repository())
        .unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, expected.stdout);
    assert_eq!(result.stderr, expected.stderr);
}
