use std::error::Error;

use crate::application::dtos::{RunAllWorkflowsRequest, RunSummary};

/// Inbound port representing the entrypoint to run all workflows in a repository.
pub trait RunAllWorkflowsPort {
    /// Executes all workflows found in the repository according to configuration.
    fn execute(&self, request: RunAllWorkflowsRequest) -> Result<RunSummary, Box<dyn Error>>;
}
