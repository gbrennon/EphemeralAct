use ephact::{
    application::dtos::ListActionsRequest,
    domain::{RepoPath, Repository, RepositoryName},
    presentation::cli::parse_list_actions_test_args,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors how `ListActionsArgs::to_domain` builds its repository from the default `.`
    /// argument. `RepoPath::new` canonicalizes, so the expected value must go through the
    /// same domain constructors. Cargo runs integration tests from the crate root, which
    /// contains a `.git` entry.
    fn current_dir_repository() -> Repository {
        let repo_path = RepoPath::new(".").unwrap();
        let name = RepositoryName::from_repo_path(&repo_path).unwrap();
        Repository::new(repo_path, name)
    }

    #[test]
    fn to_domain_returns_ok() {
        let args = parse_list_actions_test_args(&[]);

        let result = args.to_domain();

        let expected = ListActionsRequest::new(current_dir_repository());
        assert_eq!(result.unwrap(), expected);
    }
}
