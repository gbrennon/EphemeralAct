use std::time::Duration;

use ephact::application::{
    dtos::{JobSummary, MergeRunExecutionsRequest, StepSummary, StepType, WorkflowExecution},
    ports::outbound::merge_run_executions_port::MergeRunExecutionsPort,
    services::merge_run_executions_service::MergeRunExecutionsService,
};

fn job(job_id: &str, name: Option<&str>, success: bool) -> JobSummary {
    JobSummary {
        job_id: job_id.to_string(),
        name: name.map(str::to_string),
        steps: vec![StepSummary {
            name: "step".into(),
            step_type: StepType::Run,
            exit_code: Some(0),
            continue_on_error: false,
            duration: Duration::from_secs(0),
            stdout: String::new(),
            stderr: String::new(),
        }],
        success,
    }
}

fn execution(
    name: &str,
    jobs: Vec<JobSummary>,
    containers: &[&str],
    success: bool,
) -> WorkflowExecution {
    WorkflowExecution {
        workflow_name: name.to_string(),
        job_summaries: jobs,
        container_names: containers.iter().map(|c| c.to_string()).collect(),
        success,
    }
}

#[test]
fn execute_returns_a_single_run_unchanged() {
    let merged = MergeRunExecutionsService::new()
        .execute(MergeRunExecutionsRequest {
            executions: vec![execution(
                "ci",
                vec![job("build", Some("Build"), true)],
                &["container-build"],
                true,
            )],
            all_workflows: false,
        })
        .unwrap();

    assert_eq!(merged.workflow_name, "ci");
    assert_eq!(merged.job_summaries[0].name.as_deref(), Some("Build"));
    assert_eq!(merged.container_names, vec!["container-build".to_string()]);
    assert!(merged.success);
}

#[test]
fn execute_errors_for_a_single_run_with_no_execution() {
    let Err(error) = MergeRunExecutionsService::new().execute(MergeRunExecutionsRequest {
        executions: Vec::new(),
        all_workflows: false,
    }) else {
        panic!("merging no executions should fail");
    };

    assert_eq!(error.to_string(), "no workflow file resolved");
}

#[test]
fn execute_names_an_all_workflows_run_and_prefixes_every_job_name() {
    let merged = MergeRunExecutionsService::new()
        .execute(MergeRunExecutionsRequest {
            executions: vec![
                execution(
                    "ci",
                    vec![job("build", Some("build"), true)],
                    &["c-1"],
                    true,
                ),
                execution(
                    "release",
                    vec![job("publish", Some("publish"), true)],
                    &["c-2"],
                    true,
                ),
            ],
            all_workflows: true,
        })
        .unwrap();

    assert_eq!(merged.workflow_name, "all-workflows");
    assert_eq!(
        merged
            .job_summaries
            .iter()
            .map(|job| job.name.clone())
            .collect::<Vec<_>>(),
        vec![
            Some("ci / build".to_string()),
            Some("release / publish".to_string())
        ]
    );
    assert_eq!(
        merged.container_names,
        vec!["c-1".to_string(), "c-2".to_string()]
    );
    assert!(merged.success);
}

#[test]
fn execute_fails_an_all_workflows_run_when_any_execution_failed() {
    let merged = MergeRunExecutionsService::new()
        .execute(MergeRunExecutionsRequest {
            executions: vec![
                execution(
                    "ci",
                    vec![job("build", Some("build"), true)],
                    &["c-1"],
                    true,
                ),
                execution(
                    "release",
                    vec![job("publish", Some("publish"), false)],
                    &["c-2"],
                    false,
                ),
            ],
            all_workflows: true,
        })
        .unwrap();

    assert!(!merged.success);
}

#[test]
fn execute_leaves_an_unnamed_job_unnamed() {
    let merged = MergeRunExecutionsService::new()
        .execute(MergeRunExecutionsRequest {
            executions: vec![execution(
                "ci",
                vec![job("build", None, true)],
                &["c-1"],
                true,
            )],
            all_workflows: true,
        })
        .unwrap();

    assert_eq!(merged.job_summaries[0].name, None);
}
