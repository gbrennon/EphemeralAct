mod run;

use crate::core::ports::inbound::run_act_port::RunActUseCase;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ephemeral-act",
    about = "Run GitHub Actions locally in ephemeral repositories"
)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Run(run::RunArgs),
}

impl Cli {
    pub fn run<U: RunActUseCase>(use_case: U) -> Result<(), Box<dyn std::error::Error>> {
        let cli = Self::parse();
        match cli.command {
            Command::Run(args) => {
                if let Err(e) = args.execute(use_case) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                Ok(())
            }
        }
    }

    #[cfg(test)]
    /// Parse from a string slice — enables testing without env::args().
    /// Does NOT call `exit(1)` on workflow failure; propagates the error.
    pub(crate) fn run_from<U: RunActUseCase>(
        args: &[&str],
        use_case: U,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cli = Self::parse_from(args);
        match cli.command {
            Command::Run(args) => args.execute(use_case),
        }
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
                .map_err(|e| Box::<dyn std::error::Error>::from(e))
        }
    }

    #[test]
    fn run_from_parses_and_dispatches_success() {
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            }),
        };
        let result = Cli::run_from(&["ephemeral-act", "run"], use_case);
        assert!(result.is_ok());
    }

    #[test]
    fn run_from_parses_with_workflow_flag() {
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            }),
        };
        let result = Cli::run_from(
            &["ephemeral-act", "run", "--workflow", "ci.yml"],
            use_case,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn run_from_propagates_workflow_failure() {
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: "error".into(),
            }),
        };
        let err = Cli::run_from(&["ephemeral-act", "run"], use_case).unwrap_err();
        assert!(err.to_string().contains("workflow failed"));
    }

    #[test]
    fn run_from_propagates_use_case_error() {
        let use_case = StubUseCase {
            result: Err("use case failure".into()),
        };
        let err = Cli::run_from(&["ephemeral-act", "run"], use_case).unwrap_err();
        assert!(err.to_string().contains("use case failure"));
    }
}
