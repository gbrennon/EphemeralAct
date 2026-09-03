use crate::application::dtos::{RunActRequest, RunSummary};

/// Inbound port for executing a CI workflow in an ephemeral repository.
///
/// Implementing types coordinate workflow execution including container
/// management, step running, and summary output formatting.
pub trait RunActPort {
    /// Executes the workflow described by the request and returns the run
    /// summary.
    fn execute(&self, request: RunActRequest) -> Result<RunSummary, Box<dyn std::error::Error>>;
}
