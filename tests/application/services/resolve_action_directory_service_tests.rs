use std::path::{Path, PathBuf};

use ephact::application::{
    dtos::{ResolveActionDirectoryRequest, ResolvedActionDirectory},
    ports::inbound::resolve_action_directory_port::ResolveActionDirectoryPort,
    services::resolve_action_directory_service::ResolveActionDirectoryService,
};

use crate::common::fakes::fake_fetch_remote_action_port::FakeFetchRemoteActionPort;

fn service(fetcher: FakeFetchRemoteActionPort) -> ResolveActionDirectoryService {
    ResolveActionDirectoryService::new(Box::new(fetcher))
}

fn directory_of(resolved: ResolvedActionDirectory) -> PathBuf {
    match resolved {
        ResolvedActionDirectory::Directory(directory) => directory,
        ResolvedActionDirectory::Skipped(_) => panic!("expected a resolved directory"),
    }
}

#[test]
fn execute_resolves_a_local_reference_under_the_repository_root() {
    let resolved = service(FakeFetchRemoteActionPort::returning(PathBuf::from(
        "/cache",
    )))
    .execute(ResolveActionDirectoryRequest {
        action_ref: "./actions/greet",
        repo_path: Path::new("/repo"),
    })
    .unwrap();

    assert_eq!(directory_of(resolved), Path::new("/repo/actions/greet"));
}

#[test]
fn execute_skips_a_checkout_action_because_the_workspace_is_mounted() {
    let resolved = service(FakeFetchRemoteActionPort::returning(PathBuf::from(
        "/cache",
    )))
    .execute(ResolveActionDirectoryRequest {
        action_ref: "actions/checkout@v4",
        repo_path: Path::new("/repo"),
    })
    .unwrap();

    let ResolvedActionDirectory::Skipped(response) = resolved else {
        panic!("checkout should be skipped");
    };
    assert!(response.stdout.contains("[skipped]"), "{}", response.stdout);
    assert!(
        response.stdout.contains("/workspace"),
        "{}",
        response.stdout
    );
}

#[test]
fn execute_reports_container_actions_as_unsupported() {
    let error = service(FakeFetchRemoteActionPort::returning(PathBuf::from(
        "/cache",
    )))
    .execute(ResolveActionDirectoryRequest {
        action_ref: "docker://node:20",
        repo_path: Path::new("/repo"),
    })
    .unwrap_err();

    assert!(
        error.message.contains("cannot be executed yet"),
        "{}",
        error.message
    );
}

#[test]
fn execute_delegates_a_remote_reference_to_the_fetcher() {
    let fetcher = FakeFetchRemoteActionPort::returning(PathBuf::from("/cache/cache-v4"));

    let resolved = service(fetcher.clone())
        .execute(ResolveActionDirectoryRequest {
            action_ref: "https://data.forgejo.org/actions/cache@v4",
            repo_path: Path::new("/repo"),
        })
        .unwrap();

    assert_eq!(directory_of(resolved), Path::new("/cache/cache-v4"));
    assert_eq!(fetcher.fetched().len(), 1);
    assert_eq!(fetcher.fetched()[0].repo(), "cache");
}

#[test]
fn execute_surfaces_a_fetch_failure_as_a_step_error() {
    let error = service(FakeFetchRemoteActionPort::failing("network down"))
        .execute(ResolveActionDirectoryRequest {
            action_ref: "https://data.forgejo.org/actions/cache@v4",
            repo_path: Path::new("/repo"),
        })
        .unwrap_err();

    assert!(error.message.contains("network down"), "{}", error.message);
}

#[test]
fn execute_reports_an_unparseable_reference() {
    let error = service(FakeFetchRemoteActionPort::returning(PathBuf::from(
        "/cache",
    )))
    .execute(ResolveActionDirectoryRequest {
        action_ref: "",
        repo_path: Path::new("/repo"),
    })
    .unwrap_err();

    assert!(
        error.message.contains("invalid action reference"),
        "{}",
        error.message
    );
}
