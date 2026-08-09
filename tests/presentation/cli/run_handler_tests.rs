use std::error::Error;

use ephemeral_act::{
    core::{ports::inbound::run_act_port::RunActUseCase, shared_types::ExecutionResult},
    presentation::cli::{parse_run_test_args, run_handler::RunHandler},
};

struct StubUseCase {
    result: Result<ExecutionResult, String>,
}

impl RunActUseCase for StubUseCase {
    fn run_act(
        &self,
        _config: ephemeral_act::core::ActRunConfig,
        _repository: ephemeral_act::core::Repository,
    ) -> Result<ExecutionResult, Box<dyn Error>> {
        self.result.clone().map_err(Box::<dyn Error>::from)
    }
}

#[test]
fn handle_success() {
    let args = parse_run_test_args(&[]);
    let use_case = StubUseCase {
        result: Ok(ExecutionResult { success: true, stdout: String::new(), stderr: String::new() }),
    };
    assert!(RunHandler::handle(args, &use_case).is_ok());
}

#[test]
fn handle_propagates_workflow_failure() {
    let args = parse_run_test_args(&[]);
    let use_case = StubUseCase {
        result: Ok(ExecutionResult { success: false, stdout: String::new(), stderr: "error".into() }),
    };
    let err = RunHandler::handle(args, &use_case).unwrap_err();
    assert!(err.to_string().contains("workflow failed"));
}

#[test]
fn handle_propagates_use_case_error() {
    let args = parse_run_test_args(&[]);
    let use_case = StubUseCase { result: Err("use case failure".into()) };
    let err = RunHandler::handle(args, &use_case).unwrap_err();
    assert!(err.to_string().contains("use case failure"));
}

#[test]
fn handle_prints_stdout_when_present() {
    let args = parse_run_test_args(&[]);
    let use_case = StubUseCase {
        result: Ok(ExecutionResult { success: true, stdout: "build output".into(), stderr: String::new() }),
    };
    assert!(RunHandler::handle(args, &use_case).is_ok());
}

#[test]
fn handle_prints_stderr_when_present() {
    let args = parse_run_test_args(&[]);
    let use_case = StubUseCase {
        result: Ok(ExecutionResult { success: true, stdout: String::new(), stderr: "warning".into() }),
    };
    assert!(RunHandler::handle(args, &use_case).is_ok());
}
