use std::{ffi::OsString, io::Write};

use clap::Parser;

use super::{cli_parser::CliParser, command::Command};
use crate::core::ports::inbound::run_act_port::RunActPort;

/// Entry point for the presentation layer.
///
/// Holds a fully-wired use case (injected via [`Cli::new`]) and exposes
/// [`run`](Cli::run) to parse CLI arguments and dispatch to the appropriate
/// handler.
pub struct Cli {
    use_case: Box<dyn RunActPort>,
}

impl Cli {
    /// Creates a new [`Cli`] backed by the given use case.
    pub fn new<U: RunActPort + 'static>(use_case: U) -> Self {
        Self {
            use_case: Box::new(use_case),
        }
    }

    /// Parses CLI arguments and dispatches to the appropriate handler.
    ///
    /// Running without arguments prints the help to stdout and exits cleanly.
    /// On workflow failure the error is returned to the caller; `main.rs`
    /// handles printing and exiting.
    ///
    /// Accepts an explicit argument iterator so that tests can inject CLI
    /// args without touching process globals. Production code passes
    /// `std::env::args_os()`.
    pub fn run<I, T>(self, args: I) -> Result<(), Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let parsed = CliParser::try_parse_from(args);
        let cli = match parsed {
            Ok(cli) => cli,
            Err(e) => {
                if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
                    let mut stdout = std::io::stdout();
                    let _ = write!(stdout, "{e}");
                    let _ = stdout.flush();
                    return Ok(());
                }
                let _ = write!(std::io::stderr(), "{e}");
                return Err(e.to_string().into());
            }
        };
        match cli.command {
            Command::Run(args) => super::run_handler::RunHandler::handle(*args, &*self.use_case),
        }
    }
}
