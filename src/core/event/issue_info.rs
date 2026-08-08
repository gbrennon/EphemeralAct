use serde::Serialize;

use super::{label_info::LabelInfo, user_info::UserInfo};

/// Issue information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IssueInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: UserInfo,
    pub labels: Vec<LabelInfo>,
    pub html_url: String,
}
