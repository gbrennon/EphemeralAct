use std::process::Command;

pub fn execute_act_command(args: Vec<String>) -> Result<ExecutionResult, String> {
    let output = Command::new("act")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    
    let result = ExecutionResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    };

    Ok(result)
}

pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}