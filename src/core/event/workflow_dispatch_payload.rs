use std::collections::HashMap;

use serde::Serialize;

use super::{repository_info::RepositoryInfo, user_info::UserInfo};

/// Payload for `workflow_dispatch` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowDispatchPayload {
    pub inputs: HashMap<String, String>,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
    pub workflow: String,
    pub r#ref: String,
}
