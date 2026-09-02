#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ephact::{
        core::dtos::{JobSummary, RunSummary, StepSummary, StepType},
        presentation::cli::{parse_run_test_args, run_handler::RunHandler},
    };

    use crate::common::fakes::stub_run_act_port::StubRunActPort;

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
            name: "test".into(),
            job_summaries: vec![JobSummary {
                job_id: "job".into(),
                name: None,
                steps,
                success,
            }],
            success,
            duration: Duration::ZERO,
        }
    }

    #[test]
    fn handle_success() {
        let args = parse_run_test_args(&[]);
        let port = StubRunActPort {
            result: Ok(summary(true, vec![])),
        };
        assert!(RunHandler::handle(args, &port).is_ok());
    }

    #[test]
    fn handle_propagates_workflow_failure() {
        let args = parse_run_test_args(&[]);
        let port = StubRunActPort {
            result: Ok(summary(false, vec![])),
        };
        let err = RunHandler::handle(args, &port).unwrap_err();
        assert!(err.to_string().contains("workflow failed"));
    }

    #[test]
    fn handle_propagates_port_error() {
        let args = parse_run_test_args(&[]);
        let port = StubRunActPort {
            result: Err("port failure".into()),
        };
        let err = RunHandler::handle(args, &port).unwrap_err();
        assert!(err.to_string().contains("port failure"));
    }

    #[test]
    fn render_prints_workflow_header_with_status() {
        let rendered = RunHandler::render(&summary(true, vec![]));
        assert!(
            rendered.contains("Workflow 'test': succeeded"),
            "{rendered}"
        );
    }

    #[test]
    fn render_prints_failed_workflow_status() {
        let rendered = RunHandler::render(&summary(false, vec![]));
        assert!(rendered.contains("Workflow 'test': failed"), "{rendered}");
    }

    #[test]
    fn render_relays_step_stdout_when_present() {
        let rendered = RunHandler::render(&summary(true, vec![step("build", "build output", "")]));
        assert!(rendered.contains("build output"), "{rendered}");
    }

    #[test]
    fn render_relays_step_stderr_when_present() {
        let rendered = RunHandler::render(&summary(true, vec![step("warn", "", "warning")]));
        assert!(rendered.contains("warning"), "{rendered}");
    }

    #[test]
    fn render_reports_step_kind_and_outcome() {
        let rendered = RunHandler::render(&summary(true, vec![step("build", "", "")]));
        assert!(rendered.contains("Step 'build' (run): ok"), "{rendered}");
    }

    #[test]
    fn render_reports_failed_step_exit_code() {
        let mut failed = step("test", "", "");
        failed.exit_code = Some(1);
        let rendered = RunHandler::render(&summary(true, vec![failed]));
        assert!(
            rendered.contains("Step 'test' (run): failed (exit code: 1)"),
            "{rendered}"
        );
    }
}
