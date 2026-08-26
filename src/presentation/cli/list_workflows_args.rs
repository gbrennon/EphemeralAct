use std::path::PathBuf;

use clap::Args;

use crate::core::dtos::ListWorkflowsRequest;

/// CLI arguments for the `list-workflows` subcommand.
#[derive(Args)]
pub struct ListWorkflowsArgs {
    /// Path to the repository (defaults to the current directory).
    #[arg(default_value = ".")]
    path: PathBuf,
}

impl ListWorkflowsArgs {
    /// Converts CLI arguments into the domain model: a [`ListWorkflowsRequest`].
    pub fn to_domain(&self) -> Result<ListWorkflowsRequest, Box<dyn std::error::Error>> {
        Ok(ListWorkflowsRequest::new(self.path.clone()))
    }
}
