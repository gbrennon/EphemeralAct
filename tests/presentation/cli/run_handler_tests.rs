use ephemeral_act::presentation::cli::{parse_run_test_args, run_handler::RunHandler};

#[cfg(test)]
#[path = "../../fakes/stub_use_case.rs"]
mod stub_use_case;

#[cfg(test)]
mod tests {
    use ephemeral_act::core::shared_types::ExecutionResult;
    use stub_use_case::StubUseCase;

    use super::*;

    #[test]
    fn handle_success() {
        let args = parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            }),
        };
        assert!(RunHandler::handle(args, &use_case).is_ok());
    }

    #[test]
    fn handle_propagates_workflow_failure() {
        let args = parse_run_test_args(&[]);
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
        let args = parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Err("use case failure".into()),
        };
        let err = RunHandler::handle(args, &use_case).unwrap_err();
        assert!(err.to_string().contains("use case failure"));
    }

    #[test]
    fn handle_prints_stdout_when_present() {
        let args = parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: "build output".into(),
                stderr: String::new(),
            }),
        };
        assert!(RunHandler::handle(args, &use_case).is_ok());
    }

    #[test]
    fn handle_prints_stderr_when_present() {
        let args = parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(ExecutionResult {
                success: true,
                stdout: String::new(),
                stderr: "warning".into(),
            }),
        };
        assert!(RunHandler::handle(args, &use_case).is_ok());
    }
}
