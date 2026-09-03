use serde::Serialize;

use super::user_info::UserInfo;

/// Repository information included in event payloads.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RepositoryInfo {
    pub name: String,
    pub full_name: String,
    pub owner: UserInfo,
    pub private: bool,
    pub html_url: String,
    pub default_branch: String,
    pub clone_url: String,
    pub ssh_url: String,
}
