use serde::Serialize;

use super::repository_info::RepositoryInfo;

/// Branch reference for pull requests.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BranchRef {
    pub r#ref: String,
    pub sha: String,
    pub repo: RepositoryInfo,
    pub label: String,
}
