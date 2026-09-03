use std::path::Path;

use ephact::{
    application::{
        dtos::BuildRunContextRequest, ports::outbound::build_run_context_port::BuildRunContextPort,
        services::build_run_context_service::BuildRunContextService,
    },
    domain::{
        ActRunConfig, RepoPath, Repository, RepositoryName,
        value_objects::{ActEvent, ActInput, Secret},
    },
};

fn repository(path: &Path) -> Repository {
    Repository::new(
        RepoPath::new(path.to_path_buf()).unwrap(),
        RepositoryName::new("test-repo".into()).unwrap(),
    )
}

fn context(config: ActRunConfig) -> ephact::domain::expression::EvalContext {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    let repo = repository(tmp.path());
    BuildRunContextService::new().execute(BuildRunContextRequest {
        config: &config,
        repository: &repo,
    })
}

#[test]
fn execute_exposes_configured_secrets_under_the_secrets_context() {
    let config = ActRunConfig::new().add_secret(Secret::new("TOKEN".into(), "secret-value".into()));

    let context = context(config);

    assert_eq!(context.secrets["TOKEN"], "secret-value");
}

#[test]
fn execute_exposes_inputs_under_both_inputs_and_the_github_event() {
    let config = ActRunConfig::new().add_input(ActInput::new("mode".into(), "staging".into()));

    let context = context(config);

    assert_eq!(context.inputs["mode"], "staging");
    assert_eq!(context.github["event"]["inputs"]["mode"], "staging");
}

#[test]
fn execute_defaults_the_event_name_to_workflow_dispatch() {
    let context = context(ActRunConfig::new());

    assert_eq!(context.github["event_name"], "workflow_dispatch");
}

#[test]
fn execute_honours_the_configured_event_name() {
    let config = ActRunConfig::new().with_event(ActEvent::new("pull_request".into()));

    let context = context(config);

    assert_eq!(context.github["event_name"], "pull_request");
}

#[test]
fn execute_reports_the_repository_name_and_mounted_workspace() {
    let context = context(ActRunConfig::new());

    assert_eq!(context.github["repository"], "test-repo");
    assert_eq!(context.github["workspace"], "/workspace");
}

#[test]
fn execute_reports_the_runner_platform() {
    let context = context(ActRunConfig::new());

    assert_eq!(context.runner["os"], "Linux");
    assert_eq!(context.runner["arch"], "X64");
    assert_eq!(context.runner["temp"], "/tmp");
}
