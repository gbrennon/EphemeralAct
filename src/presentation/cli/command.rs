use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Execute a CI workflow in an ephemeral repository.
    Run(Box<super::run_args::RunArgs>),
    /// List workflows in a repository.
    ListWorkflows(Box<super::list_workflows_args::ListWorkflowsArgs>),
    /// List actions referenced in workflows.
    ListActions(Box<super::list_actions_args::ListActionsArgs>),
}
