use std::time::Duration;

use ephact::{
    application::{
        dtos::{ExecuteActionResponse, ExecutedStep, SummarizeStepRequest},
        ports::outbound::summarize_step_port::SummarizeStepPort,
        services::summarize_step_service::SummarizeStepService,
    },
    domain::{errors::StepError, workflow::Step},
};

fn step_from(yaml: &str) -> Step {
    serde_yaml::from_str(yaml).unwrap()
}

fn executed(step: Step, exit_code: i64) -> ExecutedStep {
    ExecutedStep {
        step,
        response: ExecuteActionResponse {
            exit_code,
            stdout: "out".into(),
            stderr: "err".into(),
        },
    }
}

#[test]
fn execute_reports_a_zero_exit_as_not_failing_the_job() {
    let step = step_from("run: echo hi\n");
    let interpolated = step_from("name: greet\nrun: echo hi\n");

    let summarized = SummarizeStepService::new().execute(SummarizeStepRequest {
        step: &step,
        outcome: Ok(executed(interpolated, 0)),
        duration: Duration::from_secs(1),
    });

    assert!(!summarized.fails_job);
    assert_eq!(summarized.summary.exit_code, Some(0));
    assert_eq!(summarized.summary.name, "greet");
}

#[test]
fn execute_reports_a_non_zero_exit_as_failing_the_job() {
    let step = step_from("run: exit 1\n");

    let summarized = SummarizeStepService::new().execute(SummarizeStepRequest {
        step: &step,
        outcome: Ok(executed(step.clone(), 1)),
        duration: Duration::from_secs(1),
    });

    assert!(summarized.fails_job);
    assert_eq!(summarized.summary.exit_code, Some(1));
}

#[test]
fn execute_keeps_the_job_passing_when_a_failing_step_continues_on_error() {
    let step = step_from("run: exit 1\ncontinue-on-error: true\n");

    let summarized = SummarizeStepService::new().execute(SummarizeStepRequest {
        step: &step,
        outcome: Ok(executed(step.clone(), 1)),
        duration: Duration::from_secs(1),
    });

    assert!(!summarized.fails_job);
    assert!(summarized.summary.continue_on_error);
}

#[test]
fn execute_reports_a_step_error_with_no_exit_code_and_the_raw_steps_label() {
    let step = step_from("name: raw label\nrun: echo hi\n");

    let summarized = SummarizeStepService::new().execute(SummarizeStepRequest {
        step: &step,
        outcome: Err(StepError {
            message: "boom".into(),
            stdout: "out".into(),
            stderr: "err".into(),
        }),
        duration: Duration::from_secs(1),
    });

    assert_eq!(summarized.summary.exit_code, None);
    assert_eq!(summarized.summary.stdout, "out");
    assert_eq!(summarized.summary.stderr, "step error: boom\nerr");
    assert_eq!(summarized.summary.name, "raw label");
    assert!(summarized.fails_job);
}

#[test]
fn execute_keeps_the_job_passing_when_an_erroring_step_continues_on_error() {
    let step = step_from("run: echo hi\ncontinue-on-error: true\n");

    let summarized = SummarizeStepService::new().execute(SummarizeStepRequest {
        step: &step,
        outcome: Err(StepError::new("boom")),
        duration: Duration::from_secs(1),
    });

    assert!(!summarized.fails_job);
}

#[test]
fn execute_falls_back_through_name_id_run_and_uses_for_the_label() {
    let service = SummarizeStepService::new();
    let cases = [
        ("name: named\nrun: echo hi\n", "named"),
        ("id: identified\nrun: echo hi\n", "identified"),
        ("run: echo hi\n", "echo hi"),
        ("uses: ./actions/greet\n", "./actions/greet"),
        ("with:\n  key: value\n", "unnamed step"),
    ];

    for (yaml, expected) in cases {
        let step = step_from(yaml);
        let summarized = service.execute(SummarizeStepRequest {
            step: &step,
            outcome: Err(StepError::new("boom")),
            duration: Duration::from_secs(1),
        });

        assert_eq!(summarized.summary.name, expected, "for {yaml:?}");
    }
}
