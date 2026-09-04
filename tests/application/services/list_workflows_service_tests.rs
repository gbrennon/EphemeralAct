use ephact::{
    application::{
        dtos::{ListWorkflowsRequest, WorkflowListItem},
        ports::inbound::list_workflows_port::ListWorkflowsPort,
        services::list_workflows_service::ListWorkflowsService,
    },
    domain::{RepoPath, Repository, RepositoryName},
};

use crate::common::fakes::fake_workflow_source::FakeWorkflowSource;

fn make_repo() -> Repository {
    let repo_path = RepoPath::new(env!("CARGO_MANIFEST_DIR")).unwrap();
    let name = RepositoryName::new("test-repo".into()).unwrap();
    Repository::new(repo_path, name)
}

#[test]
fn execute_returns_the_workflows_reported_by_the_source() {
    let workflows = vec![
        WorkflowListItem::new(Some("CI".into()), Some("ci.yml".into())),
        WorkflowListItem::new(Some("Release".into()), Some("release.yml".into())),
        WorkflowListItem::new(None, Some("unnamed.yml".into())),
    ];
    let source = FakeWorkflowSource::new().with_workflows(workflows.clone());
    let service = ListWorkflowsService::new(Box::new(source));
    let request = ListWorkflowsRequest::new(make_repo());

    let response = service.execute(request).unwrap();

    assert_eq!(response.workflows, workflows);
}

#[test]
fn execute_forwards_the_request_repository_to_the_source() {
    let source = FakeWorkflowSource::new();
    let service = ListWorkflowsService::new(Box::new(source.clone()));
    let repository = make_repo();
    let request = ListWorkflowsRequest::new(repository.clone());

    service.execute(request).unwrap();

    assert_eq!(source.list_workflows_calls(), vec![repository]);
}

#[test]
fn execute_returns_an_empty_response_when_the_source_finds_no_workflows() {
    let service = ListWorkflowsService::new(Box::new(FakeWorkflowSource::new()));
    let request = ListWorkflowsRequest::new(make_repo());

    let response = service.execute(request).unwrap();

    assert!(response.workflows.is_empty());
}

#[test]
fn execute_propagates_a_source_failure() {
    let source = FakeWorkflowSource::new().failing_list_workflows("cannot list workflows");
    let service = ListWorkflowsService::new(Box::new(source));
    let request = ListWorkflowsRequest::new(make_repo());

    let error = service.execute(request).unwrap_err();

    assert!(error.to_string().contains("cannot list workflows"));
}
