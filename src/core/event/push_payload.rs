use serde::Serialize;

use super::{commit_info::CommitInfo, repository_info::RepositoryInfo, user_info::UserInfo};

/// Payload for `push` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PushPayload {
    pub r#ref: String,
    pub before: String,
    pub after: String,
    pub repository: RepositoryInfo,
    pub pusher: UserInfo,
    pub sender: UserInfo,
    pub created: bool,
    pub deleted: bool,
    pub forced: bool,
    pub commits: Vec<CommitInfo>,
    pub head_commit: Option<CommitInfo>,
    pub compare: String,
}
