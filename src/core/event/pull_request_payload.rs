use serde::Serialize;

use super::{
    pull_request_info::PullRequestInfo, repository_info::RepositoryInfo, user_info::UserInfo,
};

/// Payload for `pull_request` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PullRequestPayload {
    pub action: String,
    pub number: u64,
    pub pull_request: PullRequestInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}
