use std::{collections::HashMap, path::Path, sync::Arc};

use ephact::{
    application::{
        dtos::ExecuteJobRequest, ports::inbound::execute_job_port::ExecuteJobPort,
        services::execute_job_service::ExecuteJobService,
    },
    domain::{expression::EvalContext, planner::Planner, workflow::Workflow},
    infrastructure::{
        jobs::GitHubJobEnvironmentAdapter,
        steps::{
            build_step_context_service::BuildStepContextService,
            prefix_step_path_service::PrefixStepPathService,
            summarize_step_service::SummarizeStepService,
        },
    },
};

use crate::common::fakes::{
    fake_command_bus::FakeCommandBus, fake_prepare_job_container_port::FakePrepareJobContainerPort,
    fake_read_step_exports_port::FakeReadStepExportsPort,
};

fn workflow(yaml: &str) -> Workflow {
    serde_yaml::from_str(yaml).unwrap()
}

fn service(
    preparer: FakePrepareJobContainerPort,
    command_bus: FakeCommandBus,
    exports: FakeReadStepExportsPort,
) -> ExecuteJobService {
    ExecuteJobService::new(
        Box::new(GitHubJobEnvironmentAdapter::new()),
        Box::new(preparer),
        Box::new(PrefixStepPathService::new()),
        Box::new(BuildStepContextService::new()),
        Box::new(SummarizeStepService::new()),
        Box::new(exports),
        Arc::new(command_bus),
    )
}

fn single_job_workflow(steps: &str) -> Workflow {
    workflow(&format!(
        "name: Ci\non: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n{steps}"
    ))
}

#[test]
fn execute_summarizes_a_single_run_step_and_reports_the_job_successful() {
    let wf = single_job_workflow("      - run: echo hi\n");
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];

    let execution = service(
        FakePrepareJobContainerPort::named("job-container"),
        FakeCommandBus::new(),
        FakeReadStepExportsPort::new(),
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    })
    .unwrap();

    assert_eq!(execution.job_summary.steps.len(), 1);
    assert!(execution.job_summary.success);
    assert_eq!(execution.container_name, "job-container");
}

#[test]
fn execute_publishes_one_step_command_per_step_with_the_prepared_container() {
    let wf = single_job_workflow("      - run: one\n      - run: two\n");
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];
    let command_bus = FakeCommandBus::new();

    service(
        FakePrepareJobContainerPort::named("job-container"),
        command_bus.clone(),
        FakeReadStepExportsPort::new(),
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    })
    .unwrap();

    let dispatched = command_bus.dispatched_steps.lock();
    assert_eq!(dispatched.len(), 2);
    assert_eq!(dispatched[0].repo_path, Path::new("/repo"));
}

#[test]
fn execute_fails_the_job_but_still_runs_the_later_steps() {
    let wf = single_job_workflow("      - run: exit 1\n      - run: echo after\n");
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];
    let command_bus = FakeCommandBus::new().queueing_step_exit_codes(vec![1, 0]);

    let execution = service(
        FakePrepareJobContainerPort::named("job-container"),
        command_bus.clone(),
        FakeReadStepExportsPort::new(),
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    })
    .unwrap();

    assert!(!execution.job_summary.success);
    assert_eq!(execution.job_summary.steps.len(), 2);
    assert_eq!(command_bus.dispatched_steps.lock().len(), 2);
}

#[test]
fn execute_passes_the_workflow_and_job_environment_to_every_step() {
    let wf = workflow(
        "name: Ci\non: push\nenv:\n  MODE: workflow\njobs:\n  build:\n    runs-on: ubuntu-latest\n    env:\n      SCOPE: job\n    steps:\n      - run: echo hi\n",
    );
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];
    let command_bus = FakeCommandBus::new();

    service(
        FakePrepareJobContainerPort::named("job-container"),
        command_bus.clone(),
        FakeReadStepExportsPort::new(),
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    })
    .unwrap();

    let environments = command_bus.dispatched_step_environments();
    let env = &environments[0];
    assert_eq!(env.get("MODE").map(String::as_str), Some("workflow"));
    assert_eq!(env.get("SCOPE").map(String::as_str), Some("job"));
    assert_eq!(
        env.get("GITHUB_WORKSPACE").map(String::as_str),
        Some("/workspace")
    );
}

#[test]
fn execute_reads_the_exports_once_per_step() {
    let wf = single_job_workflow("      - run: one\n      - run: two\n");
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];
    let exports = FakeReadStepExportsPort::new();

    service(
        FakePrepareJobContainerPort::named("job-container"),
        FakeCommandBus::new(),
        exports.clone(),
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    })
    .unwrap();

    assert_eq!(exports.calls(), 2);
}

#[test]
fn execute_carries_exported_path_additions_and_env_into_the_next_step() {
    let wf = single_job_workflow("      - run: one\n      - run: two\n");
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];
    let mut exported_env = HashMap::new();
    exported_env.insert("EXPORTED".to_string(), "yes".to_string());
    let exports =
        FakeReadStepExportsPort::queueing(vec![(vec!["/opt/bin".to_string()], exported_env)]);
    let command_bus = FakeCommandBus::new();

    service(
        FakePrepareJobContainerPort::named("job-container"),
        command_bus.clone(),
        exports,
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    })
    .unwrap();

    let environments = command_bus.dispatched_step_environments();
    let second = &environments[1];
    assert_eq!(second.get("EXPORTED").map(String::as_str), Some("yes"));
    assert!(
        second.get("PATH").unwrap().starts_with("/opt/bin:"),
        "{:?}",
        second.get("PATH")
    );
}

#[test]
fn execute_propagates_a_container_preparation_failure() {
    let wf = single_job_workflow("      - run: echo hi\n");
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];

    let Err(error) = service(
        FakePrepareJobContainerPort::failing("no runtime"),
        FakeCommandBus::new(),
        FakeReadStepExportsPort::new(),
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    }) else {
        panic!("a failing container preparation should fail the job");
    };

    assert_eq!(error.to_string(), "no runtime");
}
