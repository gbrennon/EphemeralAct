use std::{path::Path, sync::Arc};

use ephact::{
    application::{
        dtos::{ExecuteWorkflowRequest, WorkflowExecution},
        ports::inbound::execute_workflow_port::ExecuteWorkflowPort,
        services::execute_workflow_service::ExecuteWorkflowService,
    },
    domain::expression::EvalContext,
};

use crate::common::fakes::{
    fake_command_bus::FakeCommandBus, fake_load_workflow_port::FakeLoadWorkflowPort,
};

const TWO_JOBS: &str = "name: Ci\non: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: build\n  publish:\n    needs: build\n    runs-on: ubuntu-latest\n    steps:\n      - run: publish\n";

const REQUESTED_CONTENT: &str = "name: Ci\non: push\njobs: {}\n";

fn execute(
    loader: FakeLoadWorkflowPort,
    command_bus: FakeCommandBus,
) -> Result<WorkflowExecution, Box<dyn std::error::Error>> {
    ExecuteWorkflowService::new(Box::new(loader), Arc::new(command_bus)).execute(
        ExecuteWorkflowRequest {
            workflow_content: REQUESTED_CONTENT,
            repo_path: Path::new("/repo"),
            context: &EvalContext::new(),
        },
    )
}

#[test]
fn execute_publishes_a_job_command_per_job_in_dependency_order() {
    let command_bus = FakeCommandBus::new();

    let execution = execute(FakeLoadWorkflowPort::holding(TWO_JOBS), command_bus.clone()).unwrap();

    assert_eq!(command_bus.dispatched_job_ids(), vec!["build", "publish"]);
    assert_eq!(execution.job_summaries.len(), 2);
}

#[test]
fn execute_publishes_job_commands_carrying_the_loaded_workflow_and_repo_path() {
    let command_bus = FakeCommandBus::new();

    execute(FakeLoadWorkflowPort::holding(TWO_JOBS), command_bus.clone()).unwrap();

    let dispatched = command_bus.dispatched_jobs.lock();
    let first = dispatched.first().expect("a job command");
    assert_eq!(first.workflow.name.as_deref(), Some("Ci"));
    assert_eq!(first.repo_path, Path::new("/repo"));
}

#[test]
fn execute_reports_the_workflow_name() {
    let execution = execute(
        FakeLoadWorkflowPort::holding(TWO_JOBS),
        FakeCommandBus::new(),
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
        FakeCommandBus::new(),
    )
    .unwrap();

    assert_eq!(execution.workflow_name, "unnamed");
}

#[test]
fn execute_returns_every_jobs_container_name() {
    let execution = execute(
        FakeLoadWorkflowPort::holding(TWO_JOBS),
        FakeCommandBus::new(),
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
        FakeCommandBus::new().failing_jobs(vec!["publish".to_string()]),
    )
    .unwrap();

    assert!(!execution.success);
}

#[test]
fn execute_propagates_a_failed_job_dispatch() {
    let result = execute(
        FakeLoadWorkflowPort::holding(TWO_JOBS),
        FakeCommandBus::new().failing_job_dispatch("job bus is down"),
    );

    let Err(error) = result else {
        panic!("a failing job dispatch should fail the workflow");
    };
    assert_eq!(error.to_string(), "job bus is down");
}

#[test]
fn execute_errors_on_a_cyclic_dependency() {
    let cyclic = "name: Ci\non: push\njobs:\n  a:\n    needs: b\n    runs-on: ubuntu-latest\n    steps:\n      - run: a\n  b:\n    needs: a\n    runs-on: ubuntu-latest\n    steps:\n      - run: b\n";

    let result = execute(FakeLoadWorkflowPort::holding(cyclic), FakeCommandBus::new());

    assert!(result.is_err());
}

#[test]
fn execute_propagates_a_loader_error() {
    let Err(error) = execute(
        FakeLoadWorkflowPort::failing("cannot read ci.yml"),
        FakeCommandBus::new(),
    ) else {
        panic!("a failing loader should fail the workflow");
    };

    assert_eq!(error.to_string(), "cannot read ci.yml");
}
