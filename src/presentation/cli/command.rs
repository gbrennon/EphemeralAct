use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Execute a CI workflow in an ephemeral repository.
    Run(Box<super::run_args::RunArgs>),
}
