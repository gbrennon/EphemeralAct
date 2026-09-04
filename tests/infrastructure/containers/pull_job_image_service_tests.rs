use ephact::infrastructure::containers::{
    pull_job_image_port::PullJobImagePort, pull_job_image_service::PullJobImageService,
};
use std::sync::Arc;

use ephact::application::dtos::PullJobImageRequest;

use crate::common::fakes::{
    fake_image_mapper::FakeImageMapper, fake_runtime::FakeRuntime,
    stub_failing_container_runtime::StubFailingContainerRuntime,
    stub_pull_failing_runtime::StubPullFailingRuntime,
};

#[test]
fn execute_pulls_and_returns_the_mapped_image() {
    let runtime = Arc::new(FakeRuntime::new());
    let service = PullJobImageService::new(runtime.clone(), Arc::new(FakeImageMapper));

    let image = service
        .execute(PullJobImageRequest {
            runs_on: Some("ubuntu-22.04"),
        })
        .unwrap();

    assert_eq!(image, "ubuntu-22.04");
    assert_eq!(runtime.pulled_images.lock().clone(), vec!["ubuntu-22.04"]);
}

#[test]
fn execute_maps_ubuntu_latest_when_the_job_declares_no_runner() {
    let runtime = Arc::new(FakeRuntime::new());
    let service = PullJobImageService::new(runtime.clone(), Arc::new(FakeImageMapper));

    let image = service
        .execute(PullJobImageRequest { runs_on: None })
        .unwrap();

    assert_eq!(image, "ubuntu-latest");
    assert_eq!(runtime.pulled_images.lock().clone(), vec!["ubuntu-latest"]);
}

#[test]
fn execute_falls_back_to_the_mappers_default_image_when_the_first_pull_fails() {
    let runtime = Arc::new(StubPullFailingRuntime::rejecting(vec![
        "ubuntu-latest".to_string(),
    ]));
    let service = PullJobImageService::new(runtime.clone(), Arc::new(FakeImageMapper));

    let image = service
        .execute(PullJobImageRequest { runs_on: None })
        .unwrap();

    assert_eq!(image, "fake-image:latest");
    assert_eq!(
        runtime.pulled_images.lock().clone(),
        vec!["ubuntu-latest", "fake-image:latest"]
    );
}

#[test]
fn execute_errors_when_both_pulls_fail() {
    let service = PullJobImageService::new(
        Arc::new(StubFailingContainerRuntime),
        Arc::new(FakeImageMapper),
    );

    let result = service.execute(PullJobImageRequest { runs_on: None });

    assert!(result.is_err());
}
