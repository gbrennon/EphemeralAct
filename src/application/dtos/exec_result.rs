/// Result of a container exec command.
#[derive(Debug, Clone)]
pub struct ExecResult {
    /// Process exit code
    pub exit_code: i64,
    /// Captured stdout
    pub stdout: String,
    /// Captured stderr
    pub stderr: String,
}
