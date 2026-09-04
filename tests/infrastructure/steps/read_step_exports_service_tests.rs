use ephact::{
    application::ports::outbound::read_step_exports_port::ReadStepExportsPort,
    infrastructure::steps::read_step_exports_service::ReadStepExportsService,
};
use std::collections::HashMap;

use ephact::application::dtos::ReadStepExportsRequest;

use crate::common::fakes::{
    fake_read_step_env_exports_port::FakeReadStepEnvExportsPort,
    fake_read_step_path_exports_port::FakeReadStepPathExportsPort,
    stub_exporting_container::StubExportingContainer,
};

#[test]
fn execute_carries_both_collaborators_results_through() {
    let path_reader = FakeReadStepPathExportsPort::returning(vec!["/opt/bin".to_string()]);
    let mut exported = HashMap::new();
    exported.insert("A".to_string(), "1".to_string());
    let env_reader = FakeReadStepEnvExportsPort::returning(exported);
    let service =
        ReadStepExportsService::new(Box::new(path_reader.clone()), Box::new(env_reader.clone()));
    let container = StubExportingContainer::empty();

    let exports = service.execute(ReadStepExportsRequest {
        container: &container,
    });

    assert_eq!(exports.path_additions, vec!["/opt/bin".to_string()]);
    assert_eq!(exports.env.get("A").map(String::as_str), Some("1"));
    assert!(path_reader.was_called());
    assert!(env_reader.was_called());
}
