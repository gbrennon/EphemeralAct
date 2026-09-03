use std::path::Path;

use ephact::application::{
    dtos::PrepareJobContainerRequest,
    ports::outbound::prepare_job_container_port::PrepareJobContainerPort,
    services::prepare_job_container_service::PrepareJobContainerService,
};

use crate::common::fakes::{
    fake_create_job_container_port::FakeCreateJobContainerPort,
    fake_pull_job_image_port::FakePullJobImagePort,
};

fn request<'a>(repo_path: &'a Path) -> PrepareJobContainerRequest<'a> {
    PrepareJobContainerRequest {
        job_id: "build",
        runs_on: Some("ubuntu-latest"),
        repo_path,
    }
}

#[test]
fn execute_names_the_container_after_the_job_and_the_process() {
    let service = PrepareJobContainerService::new(
        Box::new(FakePullJobImagePort::returning("ubuntu:latest")),
        Box::new(FakeCreateJobContainerPort::new()),
    );

    let prepared = service.execute(request(Path::new("/repo"))).unwrap();

    assert_eq!(
        prepared.container_name,
        format!("ephemeral-act-build-{}", std::process::id())
    );
}

#[test]
fn execute_passes_the_legacy_container_name_to_the_creator() {
    let creator = FakeCreateJobContainerPort::new();
    let service = PrepareJobContainerService::new(
        Box::new(FakePullJobImagePort::returning("ubuntu:latest")),
        Box::new(creator.clone()),
    );

    service.execute(request(Path::new("/repo"))).unwrap();

    assert_eq!(
        creator.legacy_container_names(),
        vec!["ephemeral-act-build".to_string()]
    );
    assert_eq!(
        creator.container_names(),
        vec![format!("ephemeral-act-build-{}", std::process::id())]
    );
}

#[test]
fn execute_creates_the_container_from_the_pulled_image() {
    let creator = FakeCreateJobContainerPort::new();
    let service = PrepareJobContainerService::new(
        Box::new(FakePullJobImagePort::returning("mapped:image")),
        Box::new(creator.clone()),
    );

    service.execute(request(Path::new("/repo"))).unwrap();

    assert_eq!(creator.images(), vec!["mapped:image".to_string()]);
}

#[test]
fn execute_propagates_a_pull_failure() {
    let creator = FakeCreateJobContainerPort::new();
    let service = PrepareJobContainerService::new(
        Box::new(FakePullJobImagePort::failing("no such image")),
        Box::new(creator.clone()),
    );

    let Err(error) = service.execute(request(Path::new("/repo"))) else {
        panic!("a failing image pull should fail the preparation");
    };
    let error = error.to_string();

    assert_eq!(error, "no such image");
    assert!(creator.images().is_empty());
}
