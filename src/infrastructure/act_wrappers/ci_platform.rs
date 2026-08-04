use crate::core::Repository;

/// Identifies the CI platform in use by a repository.
#[derive(Debug, PartialEq)]
pub enum CiPlatform {
    GitHub,
    Forgejo,
}

impl CiPlatform {
    /// Detects the CI platform by checking which workflow directory exists.
    ///
    /// - `.forgejo/` directory present → Forgejo
    /// - `.github/workflows/` directory present → GitHub
    ///
    /// # Errors
    ///
    /// Returns an error when neither `.forgejo/` nor `.github/workflows/`
    /// exists in the repository.
    pub fn detect(repository: &Repository) -> Result<Self, String> {
        let repo_path = repository.path().as_path();
        let has_forgejo = repo_path.join(".forgejo").is_dir();
        let has_github = repo_path.join(".github").join("workflows").is_dir();

        match (has_forgejo, has_github) {
            (true, _) => Ok(CiPlatform::Forgejo),
            (false, true) => Ok(CiPlatform::GitHub),
            (false, false) => Err(format!(
                "no CI platform detected in '{}': neither .forgejo/ nor .github/workflows/ directory found",
                repo_path.display()
            )),
        }
    }
}
