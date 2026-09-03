use ephact::application::{
    dtos::ReadStepEnvExportsRequest,
    ports::outbound::read_step_env_exports_port::ReadStepEnvExportsPort,
    services::read_step_env_exports_service::ReadStepEnvExportsService,
};

use crate::common::fakes::stub_exporting_container::StubExportingContainer;

fn container(contents: &str) -> StubExportingContainer {
    StubExportingContainer::holding(vec![(
        "/workspace/.github_env".to_string(),
        contents.to_string(),
    )])
}

#[test]
fn execute_returns_the_assignments_and_skips_lines_without_an_equals() {
    let container = container("A=1\nnot-an-assignment\nB=2\n");

    let env = ReadStepEnvExportsService::new().execute(ReadStepEnvExportsRequest {
        container: &container,
    });

    assert_eq!(env.len(), 2);
    assert_eq!(env.get("A").map(String::as_str), Some("1"));
    assert_eq!(env.get("B").map(String::as_str), Some("2"));
}

#[test]
fn execute_keeps_everything_after_the_first_equals_in_the_value() {
    let container = container("QUERY=a=b=c\n");

    let env = ReadStepEnvExportsService::new().execute(ReadStepEnvExportsRequest {
        container: &container,
    });

    assert_eq!(env.get("QUERY").map(String::as_str), Some("a=b=c"));
}

#[test]
fn execute_returns_no_variables_when_the_file_was_never_written() {
    let container = StubExportingContainer::empty();

    let env = ReadStepEnvExportsService::new().execute(ReadStepEnvExportsRequest {
        container: &container,
    });

    assert!(env.is_empty());
}
