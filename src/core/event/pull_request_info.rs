use serde::Serialize;

use super::{branch_ref::BranchRef, user_info::UserInfo};

/// Pull request information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PullRequestInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub head: BranchRef,
    pub base: BranchRef,
    pub user: UserInfo,
    pub html_url: String,
    pub draft: bool,
    pub merged: bool,
    pub mergeable: Option<bool>,
}
