use serde::Serialize;

use super::{repository_info::RepositoryInfo, user_info::UserInfo};

/// Payload for `create` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CreatePayload {
    pub ref_type: String,
    pub r#ref: String,
    pub master_branch: String,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}
