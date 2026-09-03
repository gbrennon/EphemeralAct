use std::{fs, os::unix::fs::PermissionsExt};

use ephact::application::{
    dtos::CollectActionFilesRequest,
    ports::inbound::collect_action_files_port::CollectActionFilesPort,
    services::collect_action_files_service::CollectActionFilesService,
};

#[test]
fn execute_returns_files_with_action_relative_paths_and_contents() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("action.yml"), "name: Greet\n").unwrap();

    let response = CollectActionFilesService::new()
        .execute(CollectActionFilesRequest {
            action_dir: tmp.path(),
        })
        .unwrap();

    assert_eq!(response.files.len(), 1);
    assert_eq!(response.files[0].path, "action.yml");
    assert_eq!(response.files[0].content, b"name: Greet\n");
}

#[test]
fn execute_walks_nested_directories() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("dist")).unwrap();
    fs::write(tmp.path().join("dist/index.js"), "run()").unwrap();

    let response = CollectActionFilesService::new()
        .execute(CollectActionFilesRequest {
            action_dir: tmp.path(),
        })
        .unwrap();

    assert_eq!(response.files[0].path, "dist/index.js");
}

#[test]
fn execute_skips_the_git_directory() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("action.yml"), "name: Greet\n").unwrap();
    fs::create_dir_all(tmp.path().join(".git")).unwrap();
    fs::write(tmp.path().join(".git/config"), "[core]").unwrap();

    let response = CollectActionFilesService::new()
        .execute(CollectActionFilesRequest {
            action_dir: tmp.path(),
        })
        .unwrap();

    assert_eq!(response.files.len(), 1);
    assert_eq!(response.files[0].path, "action.yml");
}

#[test]
fn execute_keeps_an_executables_mode_bits() {
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("entrypoint.sh");
    fs::write(&script, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    let response = CollectActionFilesService::new()
        .execute(CollectActionFilesRequest {
            action_dir: tmp.path(),
        })
        .unwrap();

    assert_eq!(response.files[0].mode, 0o755);
}

#[test]
fn execute_errors_for_a_missing_action_directory() {
    let tmp = tempfile::tempdir().unwrap();

    let error = CollectActionFilesService::new()
        .execute(CollectActionFilesRequest {
            action_dir: &tmp.path().join("absent"),
        })
        .unwrap_err();

    assert!(
        error.message.starts_with("failed to read action directory"),
        "{}",
        error.message
    );
}
