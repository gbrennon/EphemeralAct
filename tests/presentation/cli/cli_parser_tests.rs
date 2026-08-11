use clap::Parser;
use ephemeral_act::presentation::cli::{CliParser, parse_run_test_args, run_handler::RunHandler};

#[cfg(test)]
#[path = "../../fakes/stub_use_case.rs"]
mod stub_use_case;

#[cfg(test)]
mod tests {
    use ephemeral_act::core::shared_types::ExecutionResult;
    use stub_use_case::StubUseCase;

    use super::*;

    #[test]
    fn run_dispatches_success_without_exiting() {
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            }),
        };
        let args = parse_run_test_args(&[]);
        RunHandler::handle(args, &use_case).unwrap();
    }

    #[test]
    fn run_dispatches_with_workflow_flag() {
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            }),
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
        assert_eq!(
            err.kind(),
            clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }
}
