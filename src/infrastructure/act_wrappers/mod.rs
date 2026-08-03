use crate::core::Repository;

/// Identifies the CI platform in use by a repository.
pub enum CiPlatform {
    GitHub,
    Forgejo,
}

impl CiPlatform {
    /// Detects the CI platform by checking which workflow directory exists.
    ///
    /// - `.forgejo/` directory present → Forgejo
    /// - Otherwise → GitHub (default)
    pub fn detect(repository: &Repository) -> Self {
        if repository
            .path()
            .as_path()
            .join(".forgejo")
            .is_dir()
        {
            CiPlatform::Forgejo
        } else {
            CiPlatform::GitHub
        }
    }
}

pub mod forgejo_act_wrapper;
pub mod github_act_wrapper;
