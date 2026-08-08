use std::io::Write;

use clap::Parser;

use super::{cli_parser::CliParser, command::Command};
use crate::core::ports::inbound::run_act_port::RunActUseCase;

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
                if let Err(e) = super::run_handler::RunHandler::handle(*args, &*self.use_case) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                Ok(())
            }
        }
    }
}
