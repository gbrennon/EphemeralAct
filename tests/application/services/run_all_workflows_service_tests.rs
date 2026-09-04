use std::{path::Path, sync::Arc};

use ephact::{
    application::{
        dtos::{RunAllWorkflowsRequest, RunSummary, WorkflowExecution},
        ports::inbound::RunAllWorkflowsPort,
        services::run_all_workflows_service::{ALL_WORKFLOWS_SUMMARY_NAME, RunAllWorkflowsService},
    },
    domain::{ActRunConfig, RepoPath, Repository, RepositoryName, events::DomainEvent},
};

use crate::common::fakes::{
    fake_command_bus::FakeCommandBus, fake_event_bus::FakeEventBus,
    fake_workflow_source::FakeWorkflowSource,
};

fn make_repo(path: &Path) -> Repository {
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        std::fs::create_dir_all(&git_dir).ok();
    }
    let repo_path = RepoPath::new(path.to_path_buf()).unwrap();
    let name = RepositoryName::new("test-repo".into()).unwrap();
    Repository::new(repo_path, name)
}

#[test]
fn execute_runs_all_workflows_and_merges_summary() {
    let temp = tempfile::tempdir().unwrap();
    let repo = make_repo(temp.path());

    let workflow_source = FakeWorkflowSource::new().with_all_workflow_contents(vec![
        "name: A\non: push\njobs: {}".into(),
        "name: B\non: push\njobs: {}".into(),
    ]);
    let command_bus = Arc::new(
        FakeCommandBus::new().with_workflow_result(WorkflowExecution {
            workflow_name: "TestWF".into(),
            job_summaries: Vec::new(),
            container_names: vec!["c-all".into()],
            success: true,
        }),
    );
    let event_bus = Arc::new(FakeEventBus::new());

    let service = RunAllWorkflowsService::new(
        Box::new(workflow_source),
        command_bus.clone(),
        event_bus.clone(),
    );
    let request = RunAllWorkflowsRequest::new(ActRunConfig::new(), repo);

    let summary: RunSummary = service.execute(request).unwrap();

    assert_eq!(summary.name, ALL_WORKFLOWS_SUMMARY_NAME);
    assert!(summary.success);
    assert_eq!(command_bus.dispatched_workflows.lock().len(), 2);

    let events = event_bus.events();
    assert_eq!(events.len(), 1);
    let DomainEvent::ActRunCompleted(payload) = &events[0];
    assert!(payload.success);
    assert_eq!(payload.container_names, vec!["c-all", "c-all"]);
}
