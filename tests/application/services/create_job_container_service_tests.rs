use std::{path::Path, sync::Arc};

use ephact::application::{
    dtos::CreateJobContainerRequest,
    ports::inbound::create_job_container_port::CreateJobContainerPort,
    services::create_job_container_service::CreateJobContainerService,
};

use crate::common::fakes::{
    fake_runtime::FakeRuntime, stub_failing_container_runtime::StubFailingContainerRuntime,
};

fn request<'a>(repo_path: &'a Path) -> CreateJobContainerRequest<'a> {
    CreateJobContainerRequest {
        image: "ubuntu:latest",
        container_name: "ephemeral-act-build-42",
        legacy_container_name: "ephemeral-act-build",
        repo_path,
    }
}

#[test]
fn execute_removes_the_legacy_name_then_the_current_one_before_creating() {
    let runtime = Arc::new(FakeRuntime::new());
    let service = CreateJobContainerService::new(runtime.clone());

    service.execute(request(Path::new("/repo"))).unwrap();

    assert_eq!(
        runtime.removed_containers.borrow().clone(),
        vec!["ephemeral-act-build", "ephemeral-act-build-42"]
    );
    assert_eq!(runtime.created_containers.borrow().len(), 1);
}

#[test]
fn execute_mounts_the_repository_as_the_container_workspace() {
    let runtime = Arc::new(FakeRuntime::new());
    let service = CreateJobContainerService::new(runtime.clone());

    service.execute(request(Path::new("/repo"))).unwrap();

    let created = runtime.created_containers.borrow();
    let config = created.first().unwrap();
    assert_eq!(config.image, "ubuntu:latest");
    assert_eq!(config.binds, vec!["/repo:/workspace:Z".to_string()]);
    assert_eq!(config.workdir.as_deref(), Some("/workspace"));
    assert_eq!(
        config.cmd.clone().unwrap(),
        vec!["sleep".to_string(), "infinity".to_string()]
    );
    assert_eq!(config.name.as_deref(), Some("ephemeral-act-build-42"));
}

#[test]
fn execute_errors_when_the_runtime_cannot_create_the_container() {
    let service = CreateJobContainerService::new(Arc::new(StubFailingContainerRuntime));

    let result = service.execute(request(Path::new("/repo")));

    assert!(result.is_err());
}
