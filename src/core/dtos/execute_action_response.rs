/// Response DTO for the
/// [`ExecuteActionPort`](crate::core::ports::inbound::execute_action_port::ExecuteActionPort)
/// inbound port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecuteActionResponse {
    /// Exit status of the action; non-zero fails the step.
    pub exit_code: i64,
    /// Everything the action wrote to standard output.
    pub stdout: String,
    /// Everything the action wrote to standard error.
    pub stderr: String,
}

impl ExecuteActionResponse {
    /// Creates a successful response carrying only a runner message.
    pub fn note(message: impl Into<String>) -> Self {
        Self {
            exit_code: 0,
            stdout: message.into(),
            stderr: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_succeeds_and_carries_the_message() {
        let response = ExecuteActionResponse::note("workspace already mounted\n");

        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout, "workspace already mounted\n");
        assert!(response.stderr.is_empty());
    }
}
