use std::{collections::HashMap, path::Path};

use ephact::{
    application::{
        dtos::ExecuteJobRequest,
        ports::inbound::execute_job_port::ExecuteJobPort,
        services::{
            build_step_context_service::BuildStepContextService,
            execute_job_service::ExecuteJobService,
            prefix_step_path_service::PrefixStepPathService,
            summarize_step_service::SummarizeStepService,
        },
    },
    domain::{expression::EvalContext, planner::Planner, workflow::Workflow},
    infrastructure::runners::GitHubJobEnvironmentAdapter,
};

use crate::common::fakes::{
    fake_execute_step_port::FakeExecuteStepPort,
    fake_prepare_job_container_port::FakePrepareJobContainerPort,
    fake_read_step_exports_port::FakeReadStepExportsPort,
};

fn workflow(yaml: &str) -> Workflow {
    serde_yaml::from_str(yaml).unwrap()
}

fn service(
    preparer: FakePrepareJobContainerPort,
    step_executor: FakeExecuteStepPort,
    exports: FakeReadStepExportsPort,
) -> ExecuteJobService {
    ExecuteJobService::new(
        Box::new(GitHubJobEnvironmentAdapter::new()),
        Box::new(preparer),
        Box::new(PrefixStepPathService::new()),
        Box::new(BuildStepContextService::new()),
        Box::new(step_executor),
        Box::new(SummarizeStepService::new()),
        Box::new(exports),
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
        FakeExecuteStepPort::new(),
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
fn execute_fails_the_job_but_still_runs_the_later_steps() {
    let wf = single_job_workflow("      - run: exit 1\n      - run: echo after\n");
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];
    let step_executor = FakeExecuteStepPort::queueing(vec![1, 0]);

    let execution = service(
        FakePrepareJobContainerPort::named("job-container"),
        step_executor.clone(),
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
    assert_eq!(step_executor.steps().len(), 2);
}

#[test]
fn execute_passes_the_workflow_and_job_environment_to_every_step() {
    let wf = workflow(
        "name: Ci\non: push\nenv:\n  MODE: workflow\njobs:\n  build:\n    runs-on: ubuntu-latest\n    env:\n      SCOPE: job\n    steps:\n      - run: echo hi\n",
    );
    let plan = Planner.plan(&wf).unwrap();
    let run = &plan.stages[0].runs[0];
    let step_executor = FakeExecuteStepPort::new();

    service(
        FakePrepareJobContainerPort::named("job-container"),
        step_executor.clone(),
        FakeReadStepExportsPort::new(),
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    })
    .unwrap();

    let env = &step_executor.environments()[0];
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
        FakeExecuteStepPort::new(),
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
    let step_executor = FakeExecuteStepPort::new();

    service(
        FakePrepareJobContainerPort::named("job-container"),
        step_executor.clone(),
        exports,
    )
    .execute(ExecuteJobRequest {
        run,
        workflow: &wf,
        repo_path: Path::new("/repo"),
        context: &EvalContext::new(),
    })
    .unwrap();

    let second = &step_executor.environments()[1];
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
        FakeExecuteStepPort::new(),
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
