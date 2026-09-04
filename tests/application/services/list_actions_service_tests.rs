use ephact::{
    application::{
        dtos::ListActionsRequest, ports::inbound::list_actions_port::ListActionsPort,
        services::list_actions_service::ListActionsService,
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
fn execute_returns_the_actions_reported_by_the_source() {
    let actions = vec![
        "actions/checkout@v4".to_string(),
        "./local-action".to_string(),
        "actions/setup-node@v4".to_string(),
    ];
    let source = FakeWorkflowSource::new().with_actions(actions.clone());
    let service = ListActionsService::new(Box::new(source));
    let request = ListActionsRequest::new(make_repo());

    let response = service.execute(request).unwrap();

    assert_eq!(response.actions, actions);
}

#[test]
fn execute_returns_the_actions_untransformed() {
    let actions = vec![
        "actions/checkout@v4".to_string(),
        "actions/checkout@v4".to_string(),
        "./local-action".to_string(),
    ];
    let source = FakeWorkflowSource::new().with_actions(actions.clone());
    let service = ListActionsService::new(Box::new(source));
    let request = ListActionsRequest::new(make_repo());

    let response = service.execute(request).unwrap();

    assert_eq!(response.actions, actions);
}

#[test]
fn execute_forwards_the_request_repository_to_the_source() {
    let source = FakeWorkflowSource::new();
    let service = ListActionsService::new(Box::new(source.clone()));
    let repository = make_repo();
    let request = ListActionsRequest::new(repository.clone());

    service.execute(request).unwrap();

    assert_eq!(source.list_actions_calls(), vec![repository]);
}

#[test]
fn execute_returns_an_empty_response_when_the_source_finds_no_actions() {
    let service = ListActionsService::new(Box::new(FakeWorkflowSource::new()));
    let request = ListActionsRequest::new(make_repo());

    let response = service.execute(request).unwrap();

    assert!(response.actions.is_empty());
}

#[test]
fn execute_propagates_a_source_failure() {
    let source = FakeWorkflowSource::new().failing_list_actions("cannot list actions");
    let service = ListActionsService::new(Box::new(source));
    let request = ListActionsRequest::new(make_repo());

    let error = service.execute(request).unwrap_err();

    assert!(error.to_string().contains("cannot list actions"));
}
