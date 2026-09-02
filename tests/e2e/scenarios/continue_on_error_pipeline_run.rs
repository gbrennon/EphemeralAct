use crate::{
    fakes::{failing_runtime::FailingRuntime, mirrored_action_fetcher::MirroredActionFetcher},
    support::{
        container_activity::ContainerActivity, ephemeral_act_application::EphemeralActApplication,
        workflow_repository::WorkflowRepository,
    },
};

const AUDIT_WORKFLOW: &str = r#"
name: Audit
on: schedule
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - run: echo "auditing dependencies"
        continue-on-error: true
      - run: echo "auditing licenses"
        continue-on-error: true
"#;

/// Runs a workflow whose failing steps are all marked `continue-on-error`
/// inside a container where every command fails, covering that a tolerated
/// failure keeps the run successful.
pub struct ContinueOnErrorPipelineRun {
    pub outcome: Result<(), String>,
    pub activity: ContainerActivity,
}

impl ContinueOnErrorPipelineRun {
    pub const DEPENDENCY_SCRIPT: &'static str = r#"echo "auditing dependencies""#;
    pub const LICENSE_SCRIPT: &'static str = r#"echo "auditing licenses""#;

    pub fn execute() -> Self {
        let repository =
            WorkflowRepository::named("audit-pipeline").with_workflow("audit.yml", AUDIT_WORKFLOW);
        let activity = ContainerActivity::new();
        let application = EphemeralActApplication::compose(
            FailingRuntime::recording(activity.clone()),
            MirroredActionFetcher::mirroring(repository.path()),
        );

        let outcome = application
            .cli
            .run([
                "ephemeral_act",
                "run",
                &repository.path_argument(),
                "--workflow",
                "audit.yml",
            ])
            .map_err(|error| error.to_string());

        Self { outcome, activity }
    }
}
