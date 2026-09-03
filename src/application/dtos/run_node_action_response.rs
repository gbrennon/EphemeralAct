/// Response DTO for the
/// [`RunNodeActionPort`](crate::application::ports::inbound::run_node_action_port::RunNodeActionPort)
/// outbound port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunNodeActionResponse {
    /// Process exit code.
    pub exit_code: i64,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
}
