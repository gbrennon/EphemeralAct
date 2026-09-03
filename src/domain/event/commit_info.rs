use serde::Serialize;

use super::user_info::UserInfo;

/// Commit information for push events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub timestamp: String,
    pub author: UserInfo,
    pub committer: UserInfo,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}
