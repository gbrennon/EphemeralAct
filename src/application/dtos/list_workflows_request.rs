use crate::domain::entities::repository::Repository;

/// Request DTO for listing workflows in a repository.
///
/// Carries the domain repository only: no filesystem paths are exposed to the
/// application layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListWorkflowsRequest {
    pub repository: Repository,
}

impl ListWorkflowsRequest {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}
