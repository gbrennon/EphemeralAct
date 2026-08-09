mod common;

use crate::common::FakeRunActUseCase;
use ephemeral_act::presentation::cli::Cli;

#[test]
fn new_creates_cli_instance() {
    let use_case = FakeRunActUseCase::new(true);
    let _cli = Cli::new(use_case);
}

#[test]
fn run_no_args_displays_help() {
    let use_case = FakeRunActUseCase::new(true);
    let cli = Cli::new(use_case);
    let result = cli.run(["ephemeral-act"]);
    assert!(result.is_ok());
}

#[test]
fn run_run_subcommand_succeeds() {
    let use_case = FakeRunActUseCase::new(true);
    let cli = Cli::new(use_case);
    let result = cli.run(["ephemeral-act", "run"]);
    assert!(result.is_ok());
}

#[test]
fn run_run_subcommand_propagates_workflow_failure() {
    let use_case = FakeRunActUseCase::new(false);
    let cli = Cli::new(use_case);
    let result = cli.run(["ephemeral-act", "run"]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("workflow failed"));
}

#[test]
fn run_invalid_subcommand_returns_error() {
    let use_case = FakeRunActUseCase::new(true);
    let cli = Cli::new(use_case);
    let result = cli.run(["ephemeral-act", "nonexistent"]);
    assert!(result.is_err());
}

#[test]
fn run_invalid_flag_returns_error() {
    let use_case = FakeRunActUseCase::new(true);
    let cli = Cli::new(use_case);
    let result = cli.run(["ephemeral-act", "--nonexistent-flag"]);
    assert!(result.is_err());
}
