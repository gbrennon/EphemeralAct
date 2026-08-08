use serde::Serialize;

use super::user_info::UserInfo;

/// Comment information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommentInfo {
    pub id: u64,
    pub body: String,
    pub user: UserInfo,
    pub html_url: String,
}
