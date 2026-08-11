use serde::Serialize;

use super::{release_info::ReleaseInfo, repository_info::RepositoryInfo, user_info::UserInfo};

/// Payload for `release` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReleasePayload {
    pub action: String,
    pub release: ReleaseInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}
