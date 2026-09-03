use std::{collections::HashMap, path::Path, sync::Arc};

use ephact::{
    application::{
        dtos::{ExecuteActionResponse, ExecuteStepRequest},
        ports::{inbound::execute_step_port::ExecuteStepPort, outbound::ExecResult},
        services::execute_step_service::ExecuteStepService,
    },
    domain::{errors::StepError, expression::EvalContext, workflow::Step},
};
use serde_json::Value;

use crate::common::fakes::{
    fake_request_action_execution_port::FakeRequestActionExecutionPort,
    fake_run_shell_step_port::FakeRunShellStepPort, stub_container::StubContainer,
};

fn step_from(yaml: &str) -> Step {
    serde_yaml::from_str(yaml).unwrap()
}

fn shell_result(stdout: &str) -> ExecResult {
    ExecResult {
        exit_code: 0,
        stdout: stdout.into(),
        stderr: String::new(),
    }
}

fn action_response() -> ExecuteActionResponse {
    ExecuteActionResponse {
        exit_code: 0,
        stdout: "action\n".into(),
        stderr: String::new(),
    }
}

#[test]
fn execute_runs_a_run_step_through_the_shell_runner() {
    let shell = FakeRunShellStepPort::returning(ExecResult {
        exit_code: 3,
        stdout: "out".into(),
        stderr: "err".into(),
    });
    let service = ExecuteStepService::new(
        Box::new(FakeRequestActionExecutionPort::returning(action_response())),
        Box::new(shell.clone()),
    );
    let step = step_from("run: echo hi\n");

    let executed = service
        .execute(ExecuteStepRequest {
            step: &step,
            context: &EvalContext::new(),
            container: Arc::new(StubContainer),
            repo_path: Path::new("/repo"),
            env: &HashMap::new(),
        })
        .unwrap();

    assert_eq!(executed.response.exit_code, 3);
    assert_eq!(executed.response.stdout, "out");
    assert_eq!(executed.response.stderr, "err");
    assert_eq!(shell.steps().len(), 1);
}

#[test]
fn execute_hands_a_uses_step_to_the_action_requester() {
    let requester = FakeRequestActionExecutionPort::returning(action_response());
    let service = ExecuteStepService::new(
        Box::new(requester.clone()),
        Box::new(FakeRunShellStepPort::returning(shell_result(""))),
    );
    let step = step_from("uses: ./actions/greet\n");
    let mut env = HashMap::new();
    env.insert("MODE".to_string(), "staging".to_string());

    let executed = service
        .execute(ExecuteStepRequest {
            step: &step,
            context: &EvalContext::new(),
            container: Arc::new(StubContainer),
            repo_path: Path::new("/repo"),
            env: &env,
        })
        .unwrap();

    assert_eq!(executed.response.stdout, "action\n");
    let requests = requester.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].action_ref, "./actions/greet");
    assert_eq!(requests[0].repo_path, Path::new("/repo"));
    assert_eq!(requests[0].env, env);
}

#[test]
fn execute_resolves_expressions_before_running_the_step() {
    let shell = FakeRunShellStepPort::returning(shell_result(""));
    let service = ExecuteStepService::new(
        Box::new(FakeRequestActionExecutionPort::returning(action_response())),
        Box::new(shell.clone()),
    );
    let step = step_from("run: deploy ${{ inputs.mode }}\n");
    let mut context = EvalContext::new();
    let mut inputs = serde_json::Map::new();
    inputs.insert("mode".into(), Value::String("staging".into()));
    context.inputs = Value::Object(inputs);

    service
        .execute(ExecuteStepRequest {
            step: &step,
            context: &context,
            container: Arc::new(StubContainer),
            repo_path: Path::new("/repo"),
            env: &HashMap::new(),
        })
        .unwrap();

    assert_eq!(shell.steps()[0].run.as_deref(), Some("deploy staging"));
}

#[test]
fn execute_reports_an_interpolation_failure() {
    let service = ExecuteStepService::new(
        Box::new(FakeRequestActionExecutionPort::returning(action_response())),
        Box::new(FakeRunShellStepPort::returning(shell_result(""))),
    );
    let step = step_from("run: deploy ${{ }}\n");

    let error = service
        .execute(ExecuteStepRequest {
            step: &step,
            context: &EvalContext::new(),
            container: Arc::new(StubContainer),
            repo_path: Path::new("/repo"),
            env: &HashMap::new(),
        })
        .unwrap_err();

    assert!(
        error.message.starts_with("failed to resolve expressions:"),
        "{}",
        error.message
    );
}

#[test]
fn execute_propagates_a_collaborator_error_unchanged() {
    let service = ExecuteStepService::new(
        Box::new(FakeRequestActionExecutionPort::returning(action_response())),
        Box::new(FakeRunShellStepPort::failing(StepError {
            message: "boom".into(),
            stdout: "partial".into(),
            stderr: "bad".into(),
        })),
    );
    let step = step_from("run: echo hi\n");

    let error = service
        .execute(ExecuteStepRequest {
            step: &step,
            context: &EvalContext::new(),
            container: Arc::new(StubContainer),
            repo_path: Path::new("/repo"),
            env: &HashMap::new(),
        })
        .unwrap_err();

    assert_eq!(error.message, "boom");
    assert_eq!(error.stdout, "partial");
    assert_eq!(error.stderr, "bad");
}
