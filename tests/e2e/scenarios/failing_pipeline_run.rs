use std::sync::Arc;

use crate::{
    fakes::{failing_runtime::FailingRuntime, mirrored_action_fetcher::MirroredActionFetcher},
    support::{
        container_activity::ContainerActivity, ephact_application::EphactApplication,
        workflow_repository::WorkflowRepository,
    },
};

const RELEASE_WORKFLOW: &str = r#"
name: Release
on: workflow_dispatch
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "running the suite"
  release:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: ./.forgejo/actions/release
"#;

const RELEASE_ACTION: &str = r#"
name: Release
description: Publishes the release
runs:
  using: composite
  steps:
    - run: echo "releasing"
"#;

/// Runs a workflow inside a container where every command exits with a failure
/// status, covering how a failed shell step and a failed composite action step
/// surface to the caller.
pub struct FailingPipelineRun {
    pub outcome: Result<(), String>,
    pub activity: ContainerActivity,
}

impl FailingPipelineRun {
    pub const SUITE_SCRIPT: &'static str = r#"echo "running the suite""#;
    pub const RELEASE_SCRIPT: &'static str = r#"echo "releasing""#;

    pub fn execute() -> Self {
        let repository = WorkflowRepository::named("release-pipeline")
            .with_workflow("release.yml", RELEASE_WORKFLOW)
            .with_action(".forgejo/actions/release", RELEASE_ACTION);
        let activity = ContainerActivity::new();
        let workflow_source = Arc::new(
            crate::common::fakes::fake_workflow_source::FakeWorkflowSource::new()
                .with_workflow_content(RELEASE_WORKFLOW),
        );
        let application = EphactApplication::compose(
            Arc::new(FailingRuntime::recording(activity.clone())),
            Box::new(MirroredActionFetcher::mirroring(repository.path())),
            workflow_source,
        );

        let outcome = application
            .cli
            .run([
                "ephact",
                "run",
                &repository.path_argument(),
                "--workflow",
                "release.yml",
            ])
            .map_err(|error| error.to_string());

        Self { outcome, activity }
    }
}
