use ephact::{
    application::ports::outbound::run_composite_action_port::RunCompositeActionPort,
    infrastructure::actions::run_composite_action_service::RunCompositeActionService,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ephact::application::dtos::ExecResult;
use ephact::application::dtos::{ExecuteActionRequest, RunCompositeActionRequest};
use ephact::domain::{errors::StepError, expression::EvalContext, workflow::Step};

use crate::common::fakes::{
    fake_run_composite_step_port::FakeRunCompositeStepPort, stub_container::StubContainer,
};

fn steps(yaml: &str) -> Vec<Step> {
    serde_yaml::from_str(yaml).unwrap()
}

fn action_request() -> ExecuteActionRequest {
    ExecuteActionRequest {
        action_ref: "./actions/outer".into(),
        step: serde_yaml::from_str("uses: ./actions/outer\n").unwrap(),
        repo_path: PathBuf::from("/repo"),
        env: HashMap::new(),
        context: EvalContext::new(),
        container: Arc::new(StubContainer),
    }
}

fn result(exit_code: i64, stdout: &str) -> ExecResult {
    ExecResult {
        exit_code,
        stdout: stdout.into(),
        stderr: String::new(),
    }
}

#[test]
fn execute_runs_every_step_and_concatenates_their_output() {
    let runner = FakeRunCompositeStepPort::queueing(vec![result(0, "one"), result(0, "two")]);
    let service = RunCompositeActionService::new(Box::new(runner.clone()));
    let request_owner = action_request();

    let response = service
        .execute(RunCompositeActionRequest {
            steps: &steps("- run: one\n- run: two\n"),
            inputs: &HashMap::new(),
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request_owner,
        })
        .unwrap();

    assert_eq!(response.exit_code, 0);
    assert_eq!(response.stdout, "onetwo");
    assert_eq!(runner.steps().len(), 2);
}

#[test]
fn execute_stops_at_the_first_failing_step() {
    let runner = FakeRunCompositeStepPort::queueing(vec![result(3, "one"), result(0, "two")]);
    let service = RunCompositeActionService::new(Box::new(runner.clone()));
    let request_owner = action_request();

    let response = service
        .execute(RunCompositeActionRequest {
            steps: &steps("- run: one\n- run: two\n"),
            inputs: &HashMap::new(),
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request_owner,
        })
        .unwrap();

    assert_eq!(response.exit_code, 3);
    assert_eq!(response.stdout, "one");
    assert_eq!(runner.steps().len(), 1);
}

#[test]
fn execute_carries_earlier_output_into_a_step_error() {
    let runner = FakeRunCompositeStepPort::failing(StepError {
        message: "boom".into(),
        stdout: "partial".into(),
        stderr: "bad".into(),
    });
    let service = RunCompositeActionService::new(Box::new(runner));
    let request_owner = action_request();

    let error = service
        .execute(RunCompositeActionRequest {
            steps: &steps("- run: one\n"),
            inputs: &HashMap::new(),
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request_owner,
        })
        .unwrap_err();

    assert_eq!(error.message, "boom");
    assert_eq!(error.stdout, "partial");
    assert_eq!(error.stderr, "bad");
}

#[test]
fn execute_exposes_the_actions_inputs_to_its_steps() {
    let runner = FakeRunCompositeStepPort::queueing(vec![result(0, "")]);
    let service = RunCompositeActionService::new(Box::new(runner.clone()));
    let request_owner = action_request();
    let mut inputs = HashMap::new();
    inputs.insert("mode".to_string(), "staging".to_string());

    service
        .execute(RunCompositeActionRequest {
            steps: &steps("- run: deploy ${{ inputs.mode }}\n"),
            inputs: &inputs,
            action_dir: Path::new("/repo/actions/outer"),
            action_request: &request_owner,
        })
        .unwrap();

    assert_eq!(runner.steps()[0].run.as_deref(), Some("deploy staging"));
}
