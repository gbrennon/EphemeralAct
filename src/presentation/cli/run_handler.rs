use super::run_args::RunArgs;
use crate::core::ports::inbound::run_act_port::RunActUseCase;

/// Handles the `run` subcommand by dispatching parsed CLI arguments to the
/// application use case.
///
/// Owns the presentation concerns (stdout/stderr formatting,
/// result interpretation) so that neither the domain core nor the CLI
/// argument parser needs to know about terminal I/O or exit semantics.
pub struct RunHandler;

impl RunHandler {
    /// Executes the `run` subcommand: converts CLI args to domain objects,
    /// calls the use case, prints output, and returns an error when the
    /// workflow reports failure.
    pub fn handle(
        args: RunArgs,
        use_case: &dyn RunActUseCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (config, repository) = args.to_domain()?;
        let result = use_case.run_act(config, repository)?;

        if !result.stdout.is_empty() {
            eprintln!("{}", result.stdout);
        }
        if !result.stderr.is_empty() {
            eprintln!("{}", result.stderr);
        }
        if !result.success {
            return Err("workflow failed".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn handle_success() {
        let args = crate::presentation::cli::parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            }),
        };
        let result = RunHandler::handle(args, &use_case);
        assert!(result.is_ok());
    }

    #[test]
    fn handle_propagates_workflow_failure() {
        let args = crate::presentation::cli::parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: "error".into(),
            }),
        };
        let err = RunHandler::handle(args, &use_case).unwrap_err();
        assert!(err.to_string().contains("workflow failed"));
    }

    #[test]
    fn handle_propagates_use_case_error() {
        let args = crate::presentation::cli::parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Err("use case failure".into()),
        };
        let err = RunHandler::handle(args, &use_case).unwrap_err();
        assert!(err.to_string().contains("use case failure"));
    }

    #[test]
    fn handle_prints_stdout_when_present() {
        let args = crate::presentation::cli::parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: "build output".into(),
                stderr: String::new(),
            }),
        };
        let result = RunHandler::handle(args, &use_case);
        assert!(result.is_ok());
    }

    #[test]
    fn handle_prints_stderr_when_present() {
        let args = crate::presentation::cli::parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: String::new(),
                stderr: "warning message".into(),
            }),
        };
        let result = RunHandler::handle(args, &use_case);
        assert!(result.is_ok());
    }
}
