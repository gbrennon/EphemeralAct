use serde::Serialize;

/// Release information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub html_url: String,
}
