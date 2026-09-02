use crate::{
    fakes::{
        mirrored_action_fetcher::MirroredActionFetcher, succeeding_runtime::SucceedingRuntime,
    },
    support::{
        container_activity::ContainerActivity, ephact_application::EphactApplication,
        remote_action_mirror::RemoteActionMirror, workflow_repository::WorkflowRepository,
    },
};

const TOOLCHAIN_WORKFLOW: &str = r#"
name: Remote Toolchain
on: push
jobs:
  setup:
    runs-on: ubuntu-latest
    steps:
      - uses: https://data.forgejo.org/actions/setup-node@v4
        with:
          node-version: "20"
      - run: echo "toolchain ready for ${{ github.repository }}"
"#;

const SETUP_NODE_ACTION: &str = r#"
name: Setup Node
description: Installs a node toolchain
inputs:
  node-version:
    description: Version of node to install
    default: "18"
runs:
  using: node20
  main: index.js
"#;

const SETUP_NODE_ENTRY_POINT: &str = "console.log('setup-node');\n";

/// Runs a workflow that references an action hosted on a forge other than
/// GitHub, so the reference has to be parsed, fetched, copied into the
/// container, and executed as a JavaScript action.
pub struct RemoteActionPipelineRun {
    pub outcome: Result<(), String>,
    pub activity: ContainerActivity,
    pub fetcher: MirroredActionFetcher,
}

impl RemoteActionPipelineRun {
    pub const TOOLCHAIN_SCRIPT: &'static str = r#"echo "toolchain ready for remote-toolchain""#;
    pub const ENTRY_POINT_FILE: &'static str = "index.js";
    pub const CONTAINER_ACTIONS_ROOT: &'static str = "ephemeral-act-actions";
    pub const INPUT_VARIABLE: &'static str = "INPUT_NODE-VERSION";

    pub fn execute() -> Self {
        let repository = WorkflowRepository::named("remote-toolchain")
            .with_workflow("toolchain.yml", TOOLCHAIN_WORKFLOW);
        let mirror = RemoteActionMirror::new()
            .with_definition(SETUP_NODE_ACTION)
            .with_file(Self::ENTRY_POINT_FILE, SETUP_NODE_ENTRY_POINT);
        let activity = ContainerActivity::new();
        let fetcher = MirroredActionFetcher::mirroring(mirror.path());
        let application = EphactApplication::compose(
            SucceedingRuntime::recording(activity.clone()),
            fetcher.clone(),
        );

        let outcome = application
            .cli
            .run([
                "ephact",
                "run",
                &repository.path_argument(),
                "--workflow",
                "toolchain.yml",
                "--event",
                "push",
            ])
            .map_err(|error| error.to_string());

        Self {
            outcome,
            activity,
            fetcher,
        }
    }
}
