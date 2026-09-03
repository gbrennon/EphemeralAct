use std::path::PathBuf;

use ephact::{
    application::{
        dtos::FetchRemoteActionRequest,
        ports::outbound::fetch_remote_action_port::FetchRemoteActionPort,
        services::fetch_remote_action_service::FetchRemoteActionService,
    },
    domain::value_objects::RemoteActionReference,
};

use crate::common::fakes::{
    fake_action_fetcher::FakeActionFetcher, stub_failing_action_fetcher::StubFailingActionFetcher,
};

fn reference(directory: Option<&str>) -> RemoteActionReference {
    RemoteActionReference::new(
        "https".into(),
        "data.forgejo.org".into(),
        "actions".into(),
        "cache".into(),
        directory.map(str::to_string),
        "v4".into(),
    )
}

#[test]
fn execute_returns_the_fetched_directory() {
    let fetched = PathBuf::from("/cache/actions-cache");
    let service =
        FetchRemoteActionService::new(Box::new(FakeActionFetcher::returning(fetched.clone())));

    let directory = service
        .execute(FetchRemoteActionRequest {
            reference: &reference(None),
        })
        .unwrap();

    assert_eq!(directory, fetched);
}

#[test]
fn execute_narrows_to_the_referenced_subdirectory() {
    let fetched = PathBuf::from("/cache/actions-cache");
    let service =
        FetchRemoteActionService::new(Box::new(FakeActionFetcher::returning(fetched.clone())));

    let directory = service
        .execute(FetchRemoteActionRequest {
            reference: &reference(Some("save")),
        })
        .unwrap();

    assert_eq!(directory, fetched.join("save"));
}

#[test]
fn execute_propagates_a_fetch_failure() {
    let service = FetchRemoteActionService::new(Box::new(StubFailingActionFetcher));

    let error = service
        .execute(FetchRemoteActionRequest {
            reference: &reference(None),
        })
        .unwrap_err()
        .to_string();

    assert!(error.contains("failed to fetch action"), "{error}");
}
