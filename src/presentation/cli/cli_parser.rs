use clap::Parser;

/// CLI argument parser backed by clap.
///
/// This struct is only used internally by [`Cli`] at parse time; consumers
/// never interact with it directly.
#[derive(Parser)]
#[command(
    name = "ephemeral-act",
    about = "Run GitHub Actions locally in ephemeral repositories",
    long_about = "Runs CI workflows in an ephemeral copy of a repository using \
                  `act`. The CI host is auto-detected from the \
                  repository layout; see `run --help` for the available options.",
    arg_required_else_help = true,
    after_long_help = r#"EXAMPLES:
    ephemeral-act run
    ephemeral-act run --workflow ci.yml --job test
    ephemeral-act run --event push --secret TOKEN=abc123
    ephemeral-act run --container-engine docker

CI host from the repository layout and manages ephemeral copies internally."#
)]
pub(crate) struct CliParser {
    #[command(subcommand)]
    pub(crate) command: super::command::Command,
}

/// Parses CLI arguments for the `run` subcommand from a string slice.
///
/// Intended for use in tests — avoids depending on `std::env::args()`.
#[cfg(test)]
pub(crate) fn parse_run_test_args(args: &[&str]) -> super::run_args::RunArgs {
    let mut full: Vec<&str> = vec!["ephemeral-act", "run"];
    full.extend_from_slice(args);
    let cli = CliParser::parse_from(&full);
    match cli.command {
        super::command::Command::Run(args) => *args,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ports::inbound::run_act_port::RunActUseCase, shared_types::ExecutionResult};

    struct StubUseCase {
        result: Result<ExecutionResult, String>,
    }

    impl RunActUseCase for StubUseCase {
        fn run_act(
            &self,
            _config: crate::core::ActRunConfig,
            _repository: crate::core::Repository,
        ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
            self.result
                .clone()
                .map_err(Box::<dyn std::error::Error>::from)
        }
    }

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
        super::super::run_handler::RunHandler::handle(args, &use_case).unwrap();
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
        super::super::run_handler::RunHandler::handle(args, &use_case).unwrap();
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
