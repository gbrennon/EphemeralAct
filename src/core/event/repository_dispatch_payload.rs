use serde::Serialize;

use super::{repository_info::RepositoryInfo, user_info::UserInfo};

/// Payload for `repository_dispatch` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RepositoryDispatchPayload {
    pub action: String,
    pub client_payload: serde_json::Value,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}
