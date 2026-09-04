use ephact::infrastructure::actions::{
    copy_action_to_container_port::CopyActionToContainerPort,
    copy_action_to_container_service::CopyActionToContainerService,
};
use std::path::Path;

use ephact::application::dtos::CopyActionToContainerRequest;
use ephact::application::dtos::FileEntry;

use crate::common::fakes::{
    fake_collect_action_files_port::FakeCollectActionFilesPort,
    stub_failing_container::StubFailingContainer, stub_recording_container::StubRecordingContainer,
};

fn entry() -> FileEntry {
    FileEntry {
        path: "action.yml".into(),
        content: b"name: Greet\n".to_vec(),
        mode: 0o644,
    }
}

#[test]
fn execute_returns_the_slugged_container_directory() {
    let container = StubRecordingContainer::new();
    let service =
        CopyActionToContainerService::new(Box::new(FakeCollectActionFilesPort::returning(vec![
            entry(),
        ])));

    let directory = service
        .execute(CopyActionToContainerRequest {
            action_dir: Path::new("/repo/actions/greet"),
            container: &container,
        })
        .unwrap();

    assert_eq!(directory, "/tmp/ephemeral-act-actions/_repo_actions_greet");
}

#[test]
fn execute_creates_the_directory_before_copying_the_files() {
    let container = StubRecordingContainer::new();
    let service =
        CopyActionToContainerService::new(Box::new(FakeCollectActionFilesPort::returning(vec![
            entry(),
        ])));

    service
        .execute(CopyActionToContainerRequest {
            action_dir: Path::new("/repo/actions/greet"),
            container: &container,
        })
        .unwrap();

    assert_eq!(
        container.executed_commands()[0],
        vec![
            "mkdir".to_string(),
            "-p".to_string(),
            "/tmp/ephemeral-act-actions/_repo_actions_greet".to_string()
        ]
    );
    assert_eq!(
        container.copied_paths(),
        vec!["/tmp/ephemeral-act-actions/_repo_actions_greet".to_string()]
    );
    assert_eq!(container.copied_files()[0][0].path, "action.yml");
}

#[test]
fn execute_reports_a_failing_copy() {
    let service =
        CopyActionToContainerService::new(Box::new(FakeCollectActionFilesPort::returning(vec![
            entry(),
        ])));

    let error = service
        .execute(CopyActionToContainerRequest {
            action_dir: Path::new("/repo/actions/greet"),
            container: &StubFailingContainer,
        })
        .unwrap_err();

    assert!(
        error
            .message
            .starts_with("failed to create action directory"),
        "{}",
        error.message
    );
}

#[test]
fn execute_propagates_a_collection_failure() {
    let service = CopyActionToContainerService::new(Box::new(FakeCollectActionFilesPort::failing(
        "failed to read action directory /repo",
    )));

    let error = service
        .execute(CopyActionToContainerRequest {
            action_dir: Path::new("/repo/actions/greet"),
            container: &StubRecordingContainer::new(),
        })
        .unwrap_err();

    assert_eq!(error.message, "failed to read action directory /repo");
}
