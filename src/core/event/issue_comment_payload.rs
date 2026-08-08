use serde::Serialize;

use super::{
    comment_info::CommentInfo, issue_info::IssueInfo, repository_info::RepositoryInfo,
    user_info::UserInfo,
};

/// Payload for `issue_comment` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IssueCommentPayload {
    pub action: String,
    pub issue: IssueInfo,
    pub comment: CommentInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}
