use std::path::PathBuf;

use clap::Args;

use crate::application::dtos::ListWorkflowsRequest;
use crate::domain::{RepoPath, Repository, RepositoryName};

/// CLI arguments for the `list-workflows` command.
///
/// Maps directly to `ListWorkflowsArgs` from `clap::Args`.
#[derive(Args)]
pub struct ListWorkflowsArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
}

impl ListWorkflowsArgs {
    /// Converts CLI arguments into the domain model: a [`ListWorkflowsRequest`].
    ///
    /// This translation keeps the application layer agnostic to filesystem
    /// details by constructing a domain [`Repository`] from the path.
    pub fn to_domain(&self) -> Result<ListWorkflowsRequest, Box<dyn std::error::Error>> {
        let repo_path = RepoPath::new(self.path.clone()).map_err(|e| format!("{:?}", e))?;
        let repo_name =
            RepositoryName::from_repo_path(&repo_path).map_err(|e| format!("{:?}", e))?;
        let repository = Repository::new(repo_path, repo_name);
        Ok(ListWorkflowsRequest::new(repository))
    }
}
