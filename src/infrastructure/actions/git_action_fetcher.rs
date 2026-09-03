use std::{
    env,
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    application::ports::outbound::ActionFetcherPort,
    domain::{errors::ActionError, value_objects::RemoteActionReference},
};

/// Directory name, under the cache root, that holds fetched action trees.
const CACHE_DIRECTORY: &str = "ephemeral-act/actions";

/// Fetches remote actions with the `git` CLI, caching each
/// host/owner/repo/revision combination on disk.
///
/// The fetch is platform-agnostic: the clone URL comes from the reference, so
/// GitHub, Forgejo, Codeberg, GitLab, or an on-disk mirror are all handled by
/// the same code path. A shallow clone of the requested revision is attempted
/// first; when the revision is a commit SHA that a shallow clone cannot name,
/// the repository is cloned and the revision fetched explicitly.
pub struct GitActionFetcher {
    cache_root: PathBuf,
}

impl GitActionFetcher {
    /// Creates a fetcher that caches action trees under `cache_root`.
    pub fn new(cache_root: PathBuf) -> Self {
        Self { cache_root }
    }

    /// Creates a fetcher rooted at the user's cache directory, falling back to
    /// the system temporary directory when `XDG_CACHE_HOME` and `HOME` are both
    /// unset.
    pub fn with_default_cache_root() -> Self {
        let base = env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
            .unwrap_or_else(env::temp_dir);
        Self::new(base.join(CACHE_DIRECTORY))
    }

    fn clone_shallow(url: &str, git_ref: &str, destination: &Path) -> Result<(), String> {
        Self::run_git(&[
            "clone".into(),
            "--depth".into(),
            "1".into(),
            "--branch".into(),
            git_ref.into(),
            url.into(),
            destination.display().to_string(),
        ])
    }

    fn clone_and_checkout(url: &str, git_ref: &str, destination: &Path) -> Result<(), String> {
        let path = destination.display().to_string();
        Self::run_git(&["clone".into(), url.into(), path.clone()])?;
        Self::run_git(&["-C".into(), path.clone(), "checkout".into(), git_ref.into()])
    }

    fn run_git(args: &[String]) -> Result<(), String> {
        let output = Command::new("git")
            .args(args)
            .output()
            .map_err(|error| format!("failed to run git: {error}"))?;

        if output.status.success() {
            return Ok(());
        }
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }

    fn is_populated(directory: &Path) -> bool {
        directory
            .read_dir()
            .is_ok_and(|mut entries| entries.next().is_some())
    }
}

impl ActionFetcherPort for GitActionFetcher {
    fn fetch(&self, reference: &RemoteActionReference) -> Result<PathBuf, ActionError> {
        let destination = self.cache_root.join(reference.cache_key());
        if Self::is_populated(&destination) {
            return Ok(destination);
        }

        create_dir_all(&self.cache_root).map_err(|error| {
            ActionError::FetchFailed(format!(
                "could not create cache directory {}: {error}",
                self.cache_root.display()
            ))
        })?;

        let url = reference.clone_url();
        let shallow = Self::clone_shallow(&url, reference.git_ref(), &destination);
        if shallow.is_ok() {
            return Ok(destination);
        }

        let _ = std::fs::remove_dir_all(&destination);
        Self::clone_and_checkout(&url, reference.git_ref(), &destination).map_err(|error| {
            let shallow_error = shallow.unwrap_err();
            ActionError::FetchFailed(format!(
                "{url}@{}: {shallow_error}; {error}",
                reference.git_ref()
            ))
        })?;

        Ok(destination)
    }
}
