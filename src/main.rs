use ephemeral_act::core::services::run_act_service::RunActService;
use ephemeral_act::infrastructure::ActWrapper;
use ephemeral_act::presentation::cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = ActWrapper;
    let use_case = RunActService::new(executor);
    cli::run(use_case)
}
