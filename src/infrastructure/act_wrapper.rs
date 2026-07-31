use crate::core::ports::outbound::ActExecutor;
pub use crate::core::shared_types::ExecutionResult;
use std::process::Command;

pub struct ActWrapper;

impl ActExecutor for ActWrapper {
    fn execute(&self, args: &[String]) -> Result<ExecutionResult, String> {
        let output = Command::new("act")
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;

        let result = ExecutionResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        };

        Ok(result)
    }
}

// Keep free function for backwards compat, delegates to ActWrapper
pub fn execute_act_command(args: Vec<String>) -> Result<ExecutionResult, String> {
    ActWrapper.execute(&args)
}
