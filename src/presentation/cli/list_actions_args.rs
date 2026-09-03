use std::path::PathBuf;

use clap::Args;

use crate::application::dtos::ListActionsRequest;

/// CLI arguments for the `list-actions` subcommand.
#[derive(Args)]
pub struct ListActionsArgs {
    /// Path to the repository (defaults to the current directory).
    #[arg(default_value = ".")]
    path: PathBuf,
}

impl ListActionsArgs {
    /// Converts CLI arguments into the domain model: a [`ListActionsRequest`].
    pub fn to_domain(&self) -> Result<ListActionsRequest, Box<dyn std::error::Error>> {
        Ok(ListActionsRequest::new(self.path.clone()))
    }
}
