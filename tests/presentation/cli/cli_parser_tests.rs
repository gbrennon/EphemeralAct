use clap::Parser;
use ephemeral_act::presentation::cli::{CliParser, parse_run_test_args, run_handler::RunHandler};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ephemeral_act::core::dtos::RunSummary;

    use super::*;
    use crate::common::fakes::stub_run_act_port::StubRunActPort;

    fn ok_summary() -> RunSummary {
        RunSummary {
            name: "test".into(),
            job_summaries: vec![],
            success: true,
            duration: Duration::ZERO,
        }
    }

    #[test]
    fn run_dispatches_success_without_exiting() {
        let port = StubRunActPort {
            result: Ok(ok_summary()),
        };
        let args = parse_run_test_args(&[]);
        RunHandler::handle(args, &port).unwrap();
    }

    #[test]
    fn run_dispatches_with_workflow_flag() {
        let port = StubRunActPort {
            result: Ok(ok_summary()),
        };
        let args = parse_run_test_args(&["--workflow", "ci.yml"]);
        RunHandler::handle(args, &port).unwrap();
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
