mod run_args;
mod run_handler;

use crate::core::ports::inbound::run_act_port::RunActUseCase;
use clap::{Parser, Subcommand};
use std::io::Write;

/// CLI argument parser backed by clap.
///
/// This struct is only used internally by [`Cli`] at parse time; consumers
/// never interact with it directly.
#[derive(Parser)]
#[command(
    name = "ephemeral-act",
    about = "Run GitHub Actions locally in ephemeral repositories",
    long_about = "Runs CI workflows in an ephemeral copy of a repository using \
                  `act-ephemeral.sh`. The CI host is auto-detected from the \
                  repository layout; see `run --help` for the available options.",
    arg_required_else_help = true,
    after_long_help = r#"EXAMPLES:
    ephemeral-act run
    ephemeral-act run --workflow ci.yml --job test
    ephemeral-act run --event push --secret TOKEN=abc123
    ephemeral-act run --container-engine docker

The `run` subcommand delegates to `act-ephemeral.sh`, which auto-detects the
CI host from the repository layout and manages ephemeral copies internally."#
)]
struct CliParser {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Execute a CI workflow in an ephemeral repository.
    Run(Box<run_args::RunArgs>),
}

/// Entry point for the presentation layer.
///
/// Holds a fully-wired use case (injected via [`Cli::new`]) and exposes
/// [`run`](Cli::run) to parse CLI arguments and dispatch to the appropriate
/// handler.
pub struct Cli {
    use_case: Box<dyn RunActUseCase>,
}

impl Cli {
    /// Creates a new [`Cli`] backed by the given use case.
    pub fn new<U: RunActUseCase + 'static>(use_case: U) -> Self {
        Self {
            use_case: Box::new(use_case),
        }
    }

    /// Parses CLI arguments and dispatches to the appropriate handler.
    ///
    /// Running without arguments prints the help to stdout and exits cleanly.
    /// On workflow failure the error is printed to stderr and the process
    /// exits with code 1 (matching the behaviour of `act` / `act_runner`).
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let cli = match CliParser::try_parse() {
            Ok(cli) => cli,
            Err(e)
                if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand =>
            {
                let mut stdout = std::io::stdout();
                write!(stdout, "{e}").map_err(|io| io.to_string())?;
                stdout.flush().map_err(|io| io.to_string())?;
                return Ok(());
            }
            Err(e) => e.exit(),
        };
        match cli.command {
            Command::Run(args) => {
                if let Err(e) = run_handler::RunHandler::handle(*args, &*self.use_case) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                Ok(())
            }
        }
    }
}

/// Parses CLI arguments for the `run` subcommand from a string slice.
///
/// Intended for use in tests — avoids depending on `std::env::args()`.
#[cfg(test)]
pub(crate) fn parse_run_test_args(args: &[&str]) -> run_args::RunArgs {
    let mut full: Vec<&str> = vec!["ephemeral-act", "run"];
    full.extend_from_slice(args);
    let cli = CliParser::parse_from(&full);
    match cli.command {
        Command::Run(args) => *args,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ports::inbound::run_act_port::RunActUseCase;
    use crate::core::shared_types::ExecutionResult;

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
        run_handler::RunHandler::handle(args, &use_case).unwrap();
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
        run_handler::RunHandler::handle(args, &use_case).unwrap();
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
