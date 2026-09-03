use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ephact::{
    application::{
        dtos::{ExecuteActionRequest, ExecuteActionResponse, RunCompositeStepRequest},
        ports::outbound::run_composite_step_port::RunCompositeStepPort,
        services::{
            run_composite_step_service::RunCompositeStepService,
            run_shell_step_service::RunShellStepService,
        },
    },
    domain::{expression::EvalContext, workflow::Step},
};

use crate::common::fakes::{
    spy_nested_action_executor::SpyNestedActionExecutor,
    stub_failing_container::StubFailingContainer, stub_recording_container::StubRecordingContainer,
};

fn step_from(yaml: &str) -> Step {
    serde_yaml::from_str(yaml).unwrap()
}

fn action_request(
    container: Arc<dyn ephact::application::ports::outbound::ContainerPort>,
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

fn nested() -> SpyNestedActionExecutor {
    SpyNestedActionExecutor::returning(ExecuteActionResponse {
        exit_code: 0,
        stdout: "nested\n".into(),
        stderr: String::new(),
    })
}

#[test]
fn execute_runs_a_run_step_with_the_action_path_exposed() {
    let container = StubRecordingContainer::new();
    let request = action_request(Arc::new(container.clone()));
    let step = step_from("run: echo hi\n");
    let service = RunCompositeStepService::new(Box::new(RunShellStepService::new()));

    service
        .execute(RunCompositeStepRequest {
            step: &step,
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request,
            context: &EvalContext::new(),
            nested_executor: &nested(),
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
fn execute_hands_a_uses_step_to_the_nested_executor() {
    let container = StubRecordingContainer::new();
    let request = action_request(Arc::new(container.clone()));
    let step = step_from("uses: ./actions/inner\n");
    let nested_executor = nested();
    let service = RunCompositeStepService::new(Box::new(RunShellStepService::new()));

    let result = service
        .execute(RunCompositeStepRequest {
            step: &step,
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request,
            context: &EvalContext::new(),
            nested_executor: &nested_executor,
        })
        .unwrap();

    assert_eq!(result.stdout, "nested\n");
    let requests = nested_executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].action_ref, "./actions/inner");
    assert_eq!(requests[0].repo_path, Path::new("/repo"));
    assert!(container.executed_commands().is_empty());
}

#[test]
fn execute_propagates_a_shell_runner_failure() {
    let request = action_request(Arc::new(StubFailingContainer));
    let step = step_from("run: echo hi\n");
    let service = RunCompositeStepService::new(Box::new(RunShellStepService::new()));

    let error = service
        .execute(RunCompositeStepRequest {
            step: &step,
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request,
            context: &EvalContext::new(),
            nested_executor: &nested(),
        })
        .unwrap_err();

    assert!(error.message.contains("exec refused"), "{}", error.message);
}
