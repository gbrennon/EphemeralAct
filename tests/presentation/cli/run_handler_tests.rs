#[cfg(test)]
#[path = "../../fakes/stub_use_case.rs"]
mod stub_use_case;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ephemeral_act::{
        core::dtos::{JobSummary, RunSummary, StepSummary, StepType},
        presentation::cli::{parse_run_test_args, run_handler::RunHandler},
    };

    use crate::stub_use_case::StubUseCase;

    fn step(name: &str, stdout: &str, stderr: &str) -> StepSummary {
        StepSummary {
            name: name.into(),
            step_type: StepType::Run,
            exit_code: Some(0),
            continue_on_error: false,
            duration: Duration::ZERO,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }

    fn summary(success: bool, steps: Vec<StepSummary>) -> RunSummary {
        RunSummary {
            name: Some("test".into()),
            job_summaries: vec![JobSummary {
                job_id: "job".into(),
                name: None,
                matrix: None,
                steps,
                success,
                completed_at: None,
            }],
            success,
            total_duration: Duration::ZERO,
        }
    }

    #[test]
    fn handle_success() {
        let args = parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(summary(true, vec![])),
        };
        assert!(RunHandler::handle(args, &use_case).is_ok());
    }

    #[test]
    fn handle_propagates_workflow_failure() {
        let args = parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(summary(false, vec![])),
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
    fn handle_relays_step_stdout_when_present() {
        let args = parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(summary(true, vec![step("build", "build output", "")])),
        };
        assert!(RunHandler::handle(args, &use_case).is_ok());
    }

    #[test]
    fn handle_relays_step_stderr_when_present() {
        let args = parse_run_test_args(&[]);
        let use_case = StubUseCase {
            result: Ok(summary(true, vec![step("warn", "", "warning")])),
        };
        assert!(RunHandler::handle(args, &use_case).is_ok());
    }
}
