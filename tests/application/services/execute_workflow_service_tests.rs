use std::path::Path;

use ephact::{
    application::{
        dtos::ExecuteWorkflowRequest, ports::outbound::execute_workflow_port::ExecuteWorkflowPort,
        services::execute_workflow_service::ExecuteWorkflowService,
    },
    domain::expression::EvalContext,
};

use crate::common::fakes::{
    fake_execute_job_port::FakeExecuteJobPort, fake_load_workflow_port::FakeLoadWorkflowPort,
};

const TWO_JOBS: &str = "name: Ci\non: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: build\n  publish:\n    needs: build\n    runs-on: ubuntu-latest\n    steps:\n      - run: publish\n";

fn execute(
    loader: FakeLoadWorkflowPort,
    job_executor: FakeExecuteJobPort,
) -> Result<ephact::application::dtos::WorkflowExecution, Box<dyn std::error::Error>> {
    ExecuteWorkflowService::new(Box::new(loader), Box::new(job_executor)).execute(
        ExecuteWorkflowRequest {
            workflow_file: Path::new("ci.yml"),
            repo_path: Path::new("/repo"),
            context: &EvalContext::new(),
        },
    )
}

#[test]
fn execute_runs_the_jobs_in_the_order_their_dependencies_require() {
    let job_executor = FakeExecuteJobPort::new();

    let execution = execute(
        FakeLoadWorkflowPort::holding(TWO_JOBS),
        job_executor.clone(),
    )
    .unwrap();

    assert_eq!(job_executor.executed_job_ids(), vec!["build", "publish"]);
    assert_eq!(execution.job_summaries.len(), 2);
}

#[test]
fn execute_reports_the_workflow_name() {
    let execution = execute(
        FakeLoadWorkflowPort::holding(TWO_JOBS),
        FakeExecuteJobPort::new(),
    )
    .unwrap();

    assert_eq!(execution.workflow_name, "Ci");
}

#[test]
fn execute_names_an_unnamed_workflow_unnamed() {
    let execution = execute(
        FakeLoadWorkflowPort::holding(
            "on: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: build\n",
        ),
        FakeExecuteJobPort::new(),
    )
    .unwrap();

    assert_eq!(execution.workflow_name, "unnamed");
}

#[test]
fn execute_returns_every_jobs_container_name() {
    let execution = execute(
        FakeLoadWorkflowPort::holding(TWO_JOBS),
        FakeExecuteJobPort::new(),
    )
    .unwrap();

    assert_eq!(
        execution.container_names,
        vec![
            "container-build".to_string(),
            "container-publish".to_string()
        ]
    );
}

#[test]
fn execute_fails_the_workflow_when_one_job_fails() {
    let execution = execute(
        FakeLoadWorkflowPort::holding(TWO_JOBS),
        FakeExecuteJobPort::failing(vec!["publish".to_string()]),
    )
    .unwrap();

    assert!(!execution.success);
}

#[test]
fn execute_errors_on_a_cyclic_dependency() {
    let cyclic = "name: Ci\non: push\njobs:\n  a:\n    needs: b\n    runs-on: ubuntu-latest\n    steps:\n      - run: a\n  b:\n    needs: a\n    runs-on: ubuntu-latest\n    steps:\n      - run: b\n";

    let result = execute(
        FakeLoadWorkflowPort::holding(cyclic),
        FakeExecuteJobPort::new(),
    );

    assert!(result.is_err());
}

#[test]
fn execute_propagates_a_loader_error() {
    let Err(error) = execute(
        FakeLoadWorkflowPort::failing("cannot read ci.yml"),
        FakeExecuteJobPort::new(),
    ) else {
        panic!("a failing loader should fail the workflow");
    };

    assert_eq!(error.to_string(), "cannot read ci.yml");
}
