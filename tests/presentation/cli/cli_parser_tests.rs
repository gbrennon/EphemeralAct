use std::error::Error;

use clap::Parser;
use ephemeral_act::{
    core::{ports::inbound::run_act_port::RunActUseCase, shared_types::ExecutionResult},
    presentation::cli::{CliParser, parse_run_test_args, run_handler::RunHandler},
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
fn run_dispatches_success_without_exiting() {
    let use_case = StubUseCase {
        result: Ok(ExecutionResult { success: true, stdout: String::new(), stderr: String::new() }),
    };
    let args = parse_run_test_args(&[]);
    RunHandler::handle(args, &use_case).unwrap();
}

#[test]
fn run_dispatches_with_workflow_flag() {
    let use_case = StubUseCase {
        result: Ok(ExecutionResult { success: true, stdout: String::new(), stderr: String::new() }),
    };
    let args = parse_run_test_args(&["--workflow", "ci.yml"]);
    RunHandler::handle(args, &use_case).unwrap();
}

#[test]
fn no_args_displays_help() {
    let result = CliParser::try_parse_from(["ephemeral-act"]);
    let err = match result {
        Ok(_) => panic!("expected missing-command error"),
        Err(e) => e,
    };
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand);
}
