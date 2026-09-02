use std::{path::Path, process::Command};

use ephact::{
    core::{ports::outbound::ActionFetcherPort, value_objects::RemoteActionReference},
    infrastructure::GitActionFetcher,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn git(args: &[&str], cwd: &Path) {
        let status = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .output()
            .unwrap();
        assert!(status.status.success(), "git {args:?} failed");
    }

    /// Creates `<mirror>/<owner>/<repo>` as a git repository holding an action
    /// definition on branch `main`, standing in for a forge.
    fn publish_action(mirror: &Path, owner: &str, repo: &str, body: &str) {
        let repo_dir = mirror.join(owner).join(repo);
        std::fs::create_dir_all(&repo_dir).unwrap();
        git(&["init", "--initial-branch=main"], &repo_dir);
        std::fs::write(repo_dir.join("action.yml"), body).unwrap();
        git(&["add", "."], &repo_dir);
        git(&["commit", "-m", "add action"], &repo_dir);
    }

    fn reference(mirror: &Path, owner: &str, repo: &str, git_ref: &str) -> RemoteActionReference {
        RemoteActionReference::new(
            "file".into(),
            mirror.display().to_string(),
            owner.into(),
            repo.into(),
            None,
            git_ref.into(),
        )
    }

    #[test]
    fn fetch_clones_the_action_repository() {
        let mirror = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        publish_action(
            mirror.path(),
            "actions",
            "cache",
            "name: Cache\nruns:\n  using: composite\n  steps: []\n",
        );
        let fetcher = GitActionFetcher::new(cache.path().join("actions"));

        let fetched = fetcher
            .fetch(&reference(mirror.path(), "actions", "cache", "main"))
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(fetched.join("action.yml")).unwrap(),
            "name: Cache\nruns:\n  using: composite\n  steps: []\n"
        );
    }

    #[test]
    fn fetch_reuses_the_cached_checkout() {
        let mirror = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        publish_action(
            mirror.path(),
            "actions",
            "cache",
            "name: Cache\nruns:\n  using: composite\n  steps: []\n",
        );
        let fetcher = GitActionFetcher::new(cache.path().join("actions"));
        let reference = reference(mirror.path(), "actions", "cache", "main");

        let first = fetcher.fetch(&reference).unwrap();
        std::fs::write(first.join("marker"), "cached").unwrap();
        let second = fetcher.fetch(&reference).unwrap();

        assert_eq!(first, second);
        assert!(second.join("marker").exists(), "the clone should be reused");
    }

    #[test]
    fn fetch_checks_out_a_commit_revision() {
        let mirror = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        publish_action(
            mirror.path(),
            "actions",
            "cache",
            "name: Cache\nruns:\n  using: composite\n  steps: []\n",
        );
        let repo_dir = mirror.path().join("actions/cache");
        let revision = String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&repo_dir)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();
        let fetcher = GitActionFetcher::new(cache.path().join("actions"));

        let fetched = fetcher
            .fetch(&reference(mirror.path(), "actions", "cache", &revision))
            .unwrap();

        assert!(fetched.join("action.yml").exists());
    }

    #[test]
    fn fetch_reports_an_unknown_repository() {
        let mirror = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let fetcher = GitActionFetcher::new(cache.path().join("actions"));

        let error = fetcher
            .fetch(&reference(mirror.path(), "actions", "absent", "main"))
            .unwrap_err();

        assert!(
            format!("{error}").contains("failed to fetch action"),
            "{error}"
        );
    }

    #[test]
    fn fetch_caches_each_revision_separately() {
        let mirror = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        publish_action(
            mirror.path(),
            "actions",
            "cache",
            "name: Cache\nruns:\n  using: composite\n  steps: []\n",
        );
        let repo_dir = mirror.path().join("actions/cache");
        git(&["branch", "v4"], &repo_dir);
        let fetcher = GitActionFetcher::new(cache.path().join("actions"));

        let main = fetcher
            .fetch(&reference(mirror.path(), "actions", "cache", "main"))
            .unwrap();
        let tagged = fetcher
            .fetch(&reference(mirror.path(), "actions", "cache", "v4"))
            .unwrap();

        assert_ne!(main, tagged);
    }
}
