use ephemeral_act::core::services::run_act_service::RunActService;
use ephemeral_act::infrastructure::ActionsExecutor;
use ephemeral_act::presentation::cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = ActionsExecutor::new();
    let use_case = RunActService::new(executor);
    cli::Cli::run(use_case)
}
