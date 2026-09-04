use ephact::{
    application::dtos::ReadStepPathExportsRequest,
    infrastructure::steps::{
        read_step_path_exports_port::ReadStepPathExportsPort,
        read_step_path_exports_service::ReadStepPathExportsService,
    },
};

use crate::common::fakes::stub_exporting_container::StubExportingContainer;

#[test]
fn execute_returns_the_non_empty_trimmed_lines_of_the_path_file() {
    let container = StubExportingContainer::holding(vec![(
        "/workspace/.github_path".to_string(),
        "/opt/bin\n\n/opt/tools\n".to_string(),
    )]);

    let additions = ReadStepPathExportsService::new().execute(ReadStepPathExportsRequest {
        container: &container,
    });

    assert_eq!(additions, vec!["/opt/bin", "/opt/tools"]);
}

#[test]
fn execute_returns_no_additions_when_the_file_was_never_written() {
    let container = StubExportingContainer::empty();

    let additions = ReadStepPathExportsService::new().execute(ReadStepPathExportsRequest {
        container: &container,
    });

    assert!(additions.is_empty());
}
