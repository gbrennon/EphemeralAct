use crate::{
    fakes::{
        mirrored_action_fetcher::MirroredActionFetcher, succeeding_runtime::SucceedingRuntime,
    },
    support::{
        container_activity::ContainerActivity, ephemeral_act_application::EphemeralActApplication,
        workflow_repository::WorkflowRepository,
    },
};

const PIPELINE_WORKFLOW: &str = r#"
name: Delivery Pipeline
on: pull_request
env:
  PIPELINE: delivery
jobs:
  publish:
    needs: package
    runs-on: ubuntu-latest
    steps:
      - run: echo "publishing ${{ inputs.channel }} with ${{ secrets.REGISTRY_TOKEN }}"
  build:
    runs-on: ubuntu-latest
    env:
      STAGE: build
    steps:
      - uses: actions/checkout@v4
      - run: echo "pipeline=${{ env.PIPELINE }} stage=${{ env.STAGE }}"
      - run: echo "event=${{ github.event_name }} repo=${{ github.repository }} os=${{ runner.os }}"
  package:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: ./.forgejo/actions/package
        with:
          artifact: delivery.tar
"#;

const PACKAGE_ACTION: &str = r#"
name: Package
description: Packages the build output
inputs:
  artifact:
    description: Name of the artifact to package
    required: true
  compression:
    description: Compression algorithm to apply
    default: gzip
runs:
  using: composite
  steps:
    - run: echo "packaging ${{ inputs.artifact }} with ${{ inputs.compression }}"
    - uses: ./.forgejo/actions/checksum
      with:
        artifact: ${{ inputs.artifact }}
"#;

const CHECKSUM_ACTION: &str = r#"
name: Checksum
description: Signs the packaged artifact
inputs:
  artifact:
    description: Name of the artifact to sign
    required: true
runs:
  using: composite
  steps:
    - run: echo "checksum for ${{ inputs.artifact }} signed with ${{ secrets.REGISTRY_TOKEN }}"
"#;

/// Runs a workflow whose jobs depend on each other and whose steps cover
/// workflow and job environments, the `github`, `runner`, `inputs` and
/// `secrets` contexts, a checked-out action, a local composite action with an
/// input default, and an action nested inside that composite action.
pub struct DeliveryPipelineRun {
    pub outcome: Result<(), String>,
    pub activity: ContainerActivity,
    pub fetcher: MirroredActionFetcher,
}

impl DeliveryPipelineRun {
    pub const ENVIRONMENT_SCRIPT: &'static str = r#"echo "pipeline=delivery stage=build""#;
    pub const CONTEXT_SCRIPT: &'static str =
        r#"echo "event=pull_request repo=delivery-pipeline os=Linux""#;
    pub const PACKAGE_SCRIPT: &'static str = r#"echo "packaging delivery.tar with gzip""#;
    pub const CHECKSUM_SCRIPT: &'static str =
        r#"echo "checksum for delivery.tar signed with super-secret""#;
    pub const PUBLISH_SCRIPT: &'static str = r#"echo "publishing staging with super-secret""#;

    pub fn execute() -> Self {
        let repository = WorkflowRepository::named("delivery-pipeline")
            .with_workflow("pipeline.yml", PIPELINE_WORKFLOW)
            .with_action(".forgejo/actions/package", PACKAGE_ACTION)
            .with_action(".forgejo/actions/checksum", CHECKSUM_ACTION);
        let activity = ContainerActivity::new();
        let fetcher = MirroredActionFetcher::mirroring(repository.path());
        let application = EphemeralActApplication::compose(
            SucceedingRuntime::recording(activity.clone()),
            fetcher.clone(),
        );

        let outcome = application
            .cli
            .run([
                "ephemeral_act",
                "run",
                &repository.path_argument(),
                "--workflow",
                "pipeline.yml",
                "--event",
                "pull_request",
                "--input",
                "channel=staging",
                "--secret",
                "REGISTRY_TOKEN=super-secret",
            ])
            .map_err(|error| error.to_string());

        Self {
            outcome,
            activity,
            fetcher,
        }
    }
}
