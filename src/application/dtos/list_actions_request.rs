use crate::domain::entities::repository::Repository;

/// Request DTO for listing actions referenced across a repository's workflows.
///
/// Carries the domain repository only: no filesystem paths are exposed to the
/// application layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListActionsRequest {
    pub repository: Repository,
}

impl ListActionsRequest {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}
