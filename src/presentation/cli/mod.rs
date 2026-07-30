mod args;
mod run;

use crate::core::ports::inbound::run_act_port::RunActUseCase;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ephemeral-act",
    about = "Run GitHub Actions locally in ephemeral repositories"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run(run::RunArgs),
}

pub fn run<U: RunActUseCase>(use_case: U) -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run(args) => run::execute(args, use_case),
    }
}
