use std::{error::Error, sync::Arc};

use crate::{application::dtos::WorkflowListItem, domain::entities::repository::Repository};

/// Outbound port that supplies workflow definitions and catalogues for a repository.
///
/// The application layer states *what* it needs (a workflow definition, the list of
/// workflows, the actions in use) and never *how* it is obtained. Storage details -
/// directory layout, file extensions, parsing - live entirely in the infrastructure
/// adapter that implements this port.
pub trait WorkflowSourcePort: Send + Sync {
    /// Reads the definition of one workflow, either named explicitly or auto-detected.
    fn read_workflow(
        &self,
        repository: &Repository,
        workflow_name: Option<&str>,
    ) -> Result<String, Box<dyn Error>>;

    /// Reads the definition of every workflow the repository declares.
    fn read_all_workflows(&self, repository: &Repository) -> Result<Vec<String>, Box<dyn Error>>;

    /// Lists the unique action references used across the repository's workflows.
    fn list_actions(&self, repository: &Repository) -> Result<Vec<String>, Box<dyn Error>>;

    /// Lists a summary item for each workflow the repository declares.
    fn list_workflows(
        &self,
        repository: &Repository,
    ) -> Result<Vec<WorkflowListItem>, Box<dyn Error>>;
}

impl<T: WorkflowSourcePort + ?Sized> WorkflowSourcePort for Arc<T> {
    fn read_workflow(
        &self,
        repository: &Repository,
        workflow_name: Option<&str>,
    ) -> Result<String, Box<dyn Error>> {
        (**self).read_workflow(repository, workflow_name)
    }

    fn read_all_workflows(&self, repository: &Repository) -> Result<Vec<String>, Box<dyn Error>> {
        (**self).read_all_workflows(repository)
    }

    fn list_actions(&self, repository: &Repository) -> Result<Vec<String>, Box<dyn Error>> {
        (**self).list_actions(repository)
    }

    fn list_workflows(
        &self,
        repository: &Repository,
    ) -> Result<Vec<WorkflowListItem>, Box<dyn Error>> {
        (**self).list_workflows(repository)
    }
}
