use std::sync::Arc;

use crate::{
    fakes::{
        mirrored_action_fetcher::MirroredActionFetcher, succeeding_runtime::SucceedingRuntime,
    },
    support::{
        container_activity::ContainerActivity, ephact_application::EphactApplication,
        workflow_repository::WorkflowRepository,
    },
};

const LINT_WORKFLOW: &str = r#"
name: Lint
on: push
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - run: echo "linting ${{ github.repository }}"
"#;

const TEST_WORKFLOW: &str = r#"
name: Test
on: push
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - run: echo "unit tests"
  integration:
    needs: unit
    runs-on: ubuntu-latest
    steps:
      - run: echo "integration tests"
"#;

/// Runs every workflow file of a repository in one invocation, covering the
/// `--all-workflows` mode across two files and three jobs.
pub struct EveryWorkflowRun {
    pub outcome: Result<(), String>,
    pub activity: ContainerActivity,
}

impl EveryWorkflowRun {
    pub const LINT_SCRIPT: &'static str = r#"echo "linting every-workflow""#;
    pub const UNIT_SCRIPT: &'static str = r#"echo "unit tests""#;
    pub const INTEGRATION_SCRIPT: &'static str = r#"echo "integration tests""#;

    pub fn execute() -> Self {
        let repository = WorkflowRepository::named("every-workflow")
            .with_workflow("lint.yml", LINT_WORKFLOW)
            .with_workflow("test.yml", TEST_WORKFLOW);
        let activity = ContainerActivity::new();
        let application = EphactApplication::compose(
            Arc::new(SucceedingRuntime::recording(activity.clone())),
            Box::new(MirroredActionFetcher::mirroring(repository.path())),
        );

        let outcome = application
            .cli
            .run([
                "ephact",
                "run",
                &repository.path_argument(),
                "--all-workflows",
            ])
            .map_err(|error| error.to_string());

        Self { outcome, activity }
    }
}
