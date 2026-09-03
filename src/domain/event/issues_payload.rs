use serde::Serialize;

use super::{issue_info::IssueInfo, repository_info::RepositoryInfo, user_info::UserInfo};

/// Payload for `issues` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IssuesPayload {
    pub action: String,
    pub issue: IssueInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}
