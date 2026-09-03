use serde::Serialize;

use super::{repository_info::RepositoryInfo, user_info::UserInfo};

/// Payload for `fork` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ForkPayload {
    pub forkee: RepositoryInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}
