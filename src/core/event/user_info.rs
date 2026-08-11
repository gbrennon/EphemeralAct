use serde::Serialize;

/// User/actor information included in event payloads.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
    pub login: String,
}
