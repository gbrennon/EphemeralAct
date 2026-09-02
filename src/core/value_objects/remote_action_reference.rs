/// A remote action hosted in a git repository on any forge.
///
/// The reference is platform-agnostic: `actions/checkout@v4`,
/// `https://data.forgejo.org/actions/cache@v4` and
/// `https://gitlab.com/group/action@main` all map onto this shape. The
/// `scheme`/`host` pair identifies the forge, `owner`/`repo` the repository,
/// and `directory` the sub-directory holding `action.yml` when the action does
/// not live at the repository root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteActionReference {
    scheme: String,
    host: String,
    owner: String,
    repo: String,
    directory: Option<String>,
    git_ref: String,
}

impl RemoteActionReference {
    /// Creates a reference to an action inside a remote repository.
    ///
    /// A `file` scheme treats `host` as a directory on disk that mirrors the
    /// forge layout, which lets a run resolve actions from a local mirror
    /// without network access.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::RemoteActionReference;
    /// let reference = RemoteActionReference::new(
    ///     "https".into(),
    ///     "data.forgejo.org".into(),
    ///     "actions".into(),
    ///     "cache".into(),
    ///     None,
    ///     "v4".into(),
    /// );
    /// assert_eq!(reference.clone_url(), "https://data.forgejo.org/actions/cache");
    /// ```
    pub fn new(
        scheme: String,
        host: String,
        owner: String,
        repo: String,
        directory: Option<String>,
        git_ref: String,
    ) -> Self {
        Self {
            scheme,
            host,
            owner,
            repo,
            directory,
            git_ref,
        }
    }

    /// Returns the URL a git client can clone the action repository from.
    pub fn clone_url(&self) -> String {
        if self.scheme == "file" {
            return format!("{}/{}/{}", self.host, self.owner, self.repo);
        }
        format!(
            "{}://{}/{}/{}",
            self.scheme, self.host, self.owner, self.repo
        )
    }

    /// Returns the git revision to check out (branch, tag, or commit).
    pub fn git_ref(&self) -> &str {
        &self.git_ref
    }

    /// Returns the sub-directory inside the repository that holds the action
    /// definition, or `None` when the action lives at the repository root.
    pub fn directory(&self) -> Option<&str> {
        self.directory.as_deref()
    }

    /// Returns the forge host the action is published on.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the repository owner.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Returns the repository name.
    pub fn repo(&self) -> &str {
        &self.repo
    }

    /// Returns a filesystem-safe identifier that uniquely names this
    /// host/owner/repo/revision combination, for use as a cache directory.
    pub fn cache_key(&self) -> String {
        let raw = format!(
            "{}/{}/{}/{}",
            self.host, self.owner, self.repo, self.git_ref
        );
        raw.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forgejo_cache() -> RemoteActionReference {
        RemoteActionReference::new(
            "https".into(),
            "data.forgejo.org".into(),
            "actions".into(),
            "cache".into(),
            None,
            "v4".into(),
        )
    }

    #[test]
    fn clone_url_joins_scheme_host_owner_and_repo() {
        assert_eq!(
            forgejo_cache().clone_url(),
            "https://data.forgejo.org/actions/cache"
        );
    }

    #[test]
    fn clone_url_of_file_scheme_is_a_local_path() {
        let reference = RemoteActionReference::new(
            "file".into(),
            "/srv/mirror".into(),
            "actions".into(),
            "cache".into(),
            None,
            "main".into(),
        );

        assert_eq!(reference.clone_url(), "/srv/mirror/actions/cache");
    }

    #[test]
    fn cache_key_replaces_path_separators() {
        assert_eq!(
            forgejo_cache().cache_key(),
            "data.forgejo.org_actions_cache_v4"
        );
    }

    #[test]
    fn directory_is_absent_for_root_actions() {
        assert_eq!(forgejo_cache().directory(), None);
    }

    #[test]
    fn accessors_expose_repository_coordinates() {
        let reference = forgejo_cache();

        assert_eq!(reference.host(), "data.forgejo.org");
        assert_eq!(reference.owner(), "actions");
        assert_eq!(reference.repo(), "cache");
        assert_eq!(reference.git_ref(), "v4");
    }
}
