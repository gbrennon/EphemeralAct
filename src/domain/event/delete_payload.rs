use serde::Serialize;

use super::{repository_info::RepositoryInfo, user_info::UserInfo};

/// Payload for `delete` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DeletePayload {
    pub ref_type: String,
    pub r#ref: String,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}
