use ephact::infrastructure::steps::run_composite_step_port::RunCompositeStepPort;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ephact::{
    application::dtos::{ExecuteActionRequest, ExecuteActionResponse, RunCompositeStepRequest},
    domain::{expression::EvalContext, workflow::Step},
    infrastructure::steps::{
        run_composite_step_service::RunCompositeStepService,
        run_shell_step_service::RunShellStepService,
    },
};

use crate::common::fakes::{
    fake_command_bus::FakeCommandBus, stub_failing_container::StubFailingContainer,
    stub_recording_container::StubRecordingContainer,
};

fn step_from(yaml: &str) -> Step {
    serde_yaml::from_str(yaml).unwrap()
}

fn action_request(
    container: Arc<dyn ephact::application::ports::outbound::container_port::ContainerPort>,
) -> ExecuteActionRequest {
    ExecuteActionRequest {
        action_ref: "./actions/outer".into(),
        step: step_from("uses: ./actions/outer\n"),
        repo_path: PathBuf::from("/repo"),
        env: HashMap::new(),
        context: EvalContext::new(),
        container,
    }
}

fn action_response() -> ExecuteActionResponse {
    ExecuteActionResponse {
        exit_code: 0,
        stdout: "nested\n".into(),
        stderr: String::new(),
    }
}

fn service(command_bus: FakeCommandBus) -> RunCompositeStepService {
    RunCompositeStepService::new(Box::new(RunShellStepService::new()), Arc::new(command_bus))
}

#[test]
fn execute_runs_a_run_step_with_the_action_path_exposed() {
    let container = StubRecordingContainer::new();
    let request = action_request(Arc::new(container.clone()));
    let step = step_from("run: echo hi\n");
    let service = service(FakeCommandBus::new());

    service
        .execute(RunCompositeStepRequest {
            step: &step,
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request,
            context: &EvalContext::new(),
        })
        .unwrap();

    assert_eq!(
        container.exec_environments()[0]
            .get("GITHUB_ACTION_PATH")
            .map(String::as_str),
        Some("/repo/actions/outer")
    );
}

#[test]
fn execute_publishes_an_action_command_for_a_uses_step() {
    let container = StubRecordingContainer::new();
    let request = action_request(Arc::new(container.clone()));
    let step = step_from("uses: ./actions/inner\n");
    let command_bus = FakeCommandBus::new().with_action_result(action_response());
    let service = service(command_bus.clone());

    let result = service
        .execute(RunCompositeStepRequest {
            step: &step,
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request,
            context: &EvalContext::new(),
        })
        .unwrap();

    assert_eq!(result.stdout, "nested\n");
    let dispatched = command_bus.dispatched_actions.lock();
    assert_eq!(dispatched.len(), 1);
    assert_eq!(dispatched[0].action_ref, "./actions/inner");
    assert_eq!(dispatched[0].repo_path, Path::new("/repo"));
    assert!(container.executed_commands().is_empty());
}

#[test]
fn execute_propagates_a_shell_runner_failure() {
    let request = action_request(Arc::new(StubFailingContainer));
    let step = step_from("run: echo hi\n");
    let service = service(FakeCommandBus::new());

    let error = service
        .execute(RunCompositeStepRequest {
            step: &step,
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request,
            context: &EvalContext::new(),
        })
        .unwrap_err();

    assert!(error.message.contains("exec refused"), "{}", error.message);
}
