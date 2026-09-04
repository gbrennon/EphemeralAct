use std::fs;
use std::path::{Path, PathBuf};

use ephact::{
    application::ports::outbound::WorkflowSourcePort,
    domain::{
        entities::repository::Repository,
        value_objects::{RepoPath, RepositoryName},
    },
    infrastructure::workflows::FilesystemWorkflowSource,
};
use tempfile::TempDir;

/// `RepoPath::new` rejects any directory without a `.git` entry, so every fixture
/// repository needs one before it can be handed to the port.
fn git_repository_dir() -> TempDir {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".git")).unwrap();
    tmp
}

fn repository(root: &Path) -> Repository {
    Repository::new(
        RepoPath::new(root.to_path_buf()).unwrap(),
        RepositoryName::new("test-repo".to_string()).unwrap(),
    )
}

fn write_workflow(root: &Path, file: &str, body: &str) {
    write_workflow_in(root, ".forgejo/workflows", file, body);
}

fn write_github_workflow(root: &Path, file: &str, body: &str) {
    write_workflow_in(root, ".github/workflows", file, body);
}

fn write_workflow_in(root: &Path, dir: &str, file: &str, body: &str) {
    let workflows_dir = root.join(dir);
    fs::create_dir_all(&workflows_dir).unwrap();
    fs::write(workflows_dir.join(file), body).unwrap();
}

fn source() -> FilesystemWorkflowSource {
    FilesystemWorkflowSource::default()
}

fn names(items: &[ephact::application::dtos::WorkflowListItem]) -> Vec<String> {
    items
        .iter()
        .map(|item| item.name.clone().unwrap_or_default())
        .collect()
}

#[test]
fn list_workflows_reports_name_and_path_of_forgejo_workflow() {
    let tmp = git_repository_dir();
    write_workflow(
        tmp.path(),
        "ci.yml",
        "name: CI\non: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n",
    );

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0].name.as_deref(), Some("CI"));
    let file = workflows[0].file.as_deref().unwrap();
    assert!(
        file.ends_with(".forgejo/workflows/ci.yml"),
        "expected the full workflow path, got {file:?}"
    );
}

#[test]
fn list_workflows_finds_workflow_in_github_directory() {
    let tmp = git_repository_dir();
    write_github_workflow(
        tmp.path(),
        "build.yml",
        "name: Build\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n",
    );

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0].name.as_deref(), Some("Build"));
    let file = workflows[0].file.as_deref().unwrap();
    assert!(
        file.ends_with(".github/workflows/build.yml"),
        "expected the full workflow path, got {file:?}"
    );
}

#[test]
fn list_workflows_returns_one_item_per_workflow_file_sorted_by_path() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "deploy.yml", "name: Deploy\non: [release]\n");
    write_workflow(tmp.path(), "ci.yml", "name: CI\non: [push]\n");
    write_github_workflow(tmp.path(), "build.yml", "name: Build\non: [push]\n");

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    // Paths are sorted, and ".forgejo" sorts before ".github".
    assert_eq!(names(&workflows), vec!["CI", "Deploy", "Build"]);
}

#[test]
fn list_workflows_accepts_the_yaml_extension() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "ci.yaml", "name: CI\non: [push]\n");

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    assert_eq!(names(&workflows), vec!["CI"]);
}

#[test]
fn list_workflows_ignores_files_without_a_workflow_extension() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "README.md", "name: CI\n");
    write_workflow(tmp.path(), "Makefile", "name: CI\n");

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    assert!(workflows.is_empty());
}

#[test]
fn list_workflows_strips_quotes_from_the_workflow_name() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "ci.yml", "name: \"CI Pipeline\"\non: [push]\n");

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    assert_eq!(workflows[0].name.as_deref(), Some("CI Pipeline"));
}

#[test]
fn list_workflows_skips_workflow_file_without_a_name_key() {
    let tmp = git_repository_dir();
    write_workflow(
        tmp.path(),
        "ci.yml",
        "on: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n",
    );

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    // An item is only emitted when a name could be extracted.
    assert!(workflows.is_empty());
}

#[test]
fn list_workflows_keeps_named_workflow_when_a_sibling_file_has_no_name() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "anonymous.yml", "on: [push]\n");
    write_workflow(tmp.path(), "deploy.yml", "name: Deploy\non: [release]\n");

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    assert_eq!(names(&workflows), vec!["Deploy"]);
}

#[test]
fn list_workflows_is_empty_when_repository_has_no_workflow_directories() {
    let tmp = git_repository_dir();

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    assert!(workflows.is_empty());
}

#[test]
fn list_workflows_is_empty_when_the_workflow_directory_disappeared() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "ci.yml", "name: CI\non: [push]\n");
    let repository = repository(tmp.path());
    fs::remove_dir_all(tmp.path().join(".forgejo")).unwrap();

    let workflows = source().list_workflows(&repository).unwrap();

    // A missing workflow directory is skipped rather than reported as an error.
    assert!(workflows.is_empty());
}

#[test]
fn repository_cannot_be_built_for_a_nonexistent_path() {
    let tmp = git_repository_dir();

    // A nonexistent path never reaches the port: it is rejected while building
    // the repository, so the adapter has no invalid-path branch to exercise.
    assert!(RepoPath::new(tmp.path().join("nonexistent")).is_err());
    assert!(RepoPath::new(PathBuf::from("/nonexistent")).is_err());
}

#[test]
fn list_actions_collects_uses_references_in_both_line_forms() {
    let tmp = git_repository_dir();
    write_workflow(
        tmp.path(),
        "ci.yml",
        "jobs:\n  test:\n    steps:\n      - uses: actions/checkout@v4\n        uses: ./local-action\n",
    );

    let actions = source().list_actions(&repository(tmp.path())).unwrap();

    // Collected into a BTreeSet, so the result is sorted.
    assert_eq!(actions, vec!["./local-action", "actions/checkout@v4"]);
}

#[test]
fn list_actions_deduplicates_repeated_references() {
    let tmp = git_repository_dir();
    write_workflow(
        tmp.path(),
        "ci.yml",
        "jobs:\n  test:\n    steps:\n      - uses: actions/checkout@v4\n      - uses: actions/checkout@v4\n",
    );

    let actions = source().list_actions(&repository(tmp.path())).unwrap();

    assert_eq!(actions, vec!["actions/checkout@v4"]);
}

#[test]
fn list_actions_skips_empty_and_comment_only_values() {
    let tmp = git_repository_dir();
    write_workflow(
        tmp.path(),
        "ci.yml",
        "steps:\n  - uses:\n  - uses: # pinned elsewhere\n  - uses: actions/checkout@v4\n",
    );

    let actions = source().list_actions(&repository(tmp.path())).unwrap();

    assert_eq!(actions, vec!["actions/checkout@v4"]);
}

#[test]
fn list_actions_collects_across_both_workflow_directories() {
    let tmp = git_repository_dir();
    write_workflow(
        tmp.path(),
        "ci.yml",
        "steps:\n  - uses: actions/checkout@v4\n",
    );
    write_github_workflow(
        tmp.path(),
        "build.yml",
        "steps:\n  - uses: actions/cache@v4\n",
    );

    let actions = source().list_actions(&repository(tmp.path())).unwrap();

    assert_eq!(actions, vec!["actions/cache@v4", "actions/checkout@v4"]);
}

#[test]
fn list_actions_is_empty_when_no_uses_reference_exists() {
    let tmp = git_repository_dir();
    write_workflow(
        tmp.path(),
        "ci.yml",
        "jobs:\n  test:\n    steps:\n      - run: echo hello\n",
    );

    let actions = source().list_actions(&repository(tmp.path())).unwrap();

    assert!(actions.is_empty());
}

#[test]
fn list_actions_is_empty_when_repository_has_no_workflow_directories() {
    let tmp = git_repository_dir();

    let actions = source().list_actions(&repository(tmp.path())).unwrap();

    assert!(actions.is_empty());
}

#[test]
fn read_workflow_returns_the_content_of_the_workflow_with_the_requested_name() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "ci.yml", "name: CI\non: [push]\n");
    let deploy = "name: Deploy\non: [release]\n";
    write_workflow(tmp.path(), "deploy.yml", deploy);

    let content = source()
        .read_workflow(&repository(tmp.path()), Some("Deploy"))
        .unwrap();

    assert_eq!(content, deploy);
}

#[test]
fn read_workflow_errors_when_no_workflow_carries_the_requested_name() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "ci.yml", "name: CI\non: [push]\n");

    let error = source()
        .read_workflow(&repository(tmp.path()), Some("Deploy"))
        .unwrap_err();

    assert!(
        error.to_string().contains("not found"),
        "unexpected error: {error}"
    );
}

#[test]
fn read_workflow_matches_the_name_key_and_not_the_file_stem() {
    let tmp = git_repository_dir();
    write_workflow(tmp.path(), "ci.yml", "name: CI\non: [push]\n");

    let error = source()
        .read_workflow(&repository(tmp.path()), Some("ci.yml"))
        .unwrap_err();

    assert!(
        error.to_string().contains("not found"),
        "unexpected error: {error}"
    );
}

#[test]
fn read_workflow_without_a_name_returns_the_first_workflow_by_path() {
    let tmp = git_repository_dir();
    let ci = "name: CI\non: [push]\n";
    write_workflow(tmp.path(), "ci.yml", ci);
    write_workflow(tmp.path(), "deploy.yml", "name: Deploy\non: [release]\n");

    let content = source()
        .read_workflow(&repository(tmp.path()), None)
        .unwrap();

    assert_eq!(content, ci);
}

#[test]
fn read_workflow_without_a_name_errors_when_no_workflow_file_exists() {
    let tmp = git_repository_dir();

    let error = source()
        .read_workflow(&repository(tmp.path()), None)
        .unwrap_err();

    assert!(
        error.to_string().contains("no workflow files found"),
        "unexpected error: {error}"
    );
}

#[test]
fn read_workflow_without_a_name_reads_an_unnamed_workflow_file() {
    let tmp = git_repository_dir();
    let body = "on: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n";
    write_workflow(tmp.path(), "ci.yml", body);

    let content = source()
        .read_workflow(&repository(tmp.path()), None)
        .unwrap();

    assert_eq!(content, body);
}

#[test]
fn read_all_workflows_returns_the_content_of_every_workflow_file() {
    let tmp = git_repository_dir();
    let ci = "name: CI\non: [push]\n";
    let deploy = "name: Deploy\non: [release]\n";
    let build = "name: Build\non: [push]\n";
    write_workflow(tmp.path(), "ci.yml", ci);
    write_workflow(tmp.path(), "deploy.yml", deploy);
    write_github_workflow(tmp.path(), "build.yml", build);

    let contents = source()
        .read_all_workflows(&repository(tmp.path()))
        .unwrap();

    assert_eq!(contents, vec![ci, deploy, build]);
}

#[test]
fn read_all_workflows_includes_workflow_files_without_a_name() {
    let tmp = git_repository_dir();
    let body = "on: [push]\n";
    write_workflow(tmp.path(), "ci.yml", body);

    let contents = source()
        .read_all_workflows(&repository(tmp.path()))
        .unwrap();

    assert_eq!(contents, vec![body]);
}

#[test]
fn read_all_workflows_is_empty_when_repository_has_no_workflow_directories() {
    let tmp = git_repository_dir();

    let contents = source()
        .read_all_workflows(&repository(tmp.path()))
        .unwrap();

    assert!(contents.is_empty());
}

#[test]
fn new_discovers_workflows_in_the_configured_directories_only() {
    let tmp = git_repository_dir();
    write_workflow_in(tmp.path(), "custom/workflows", "ci.yml", "name: CI\n");
    write_workflow(tmp.path(), "ignored.yml", "name: Ignored\n");

    let workflows = FilesystemWorkflowSource::new(&["custom/workflows"])
        .list_workflows(&repository(tmp.path()))
        .unwrap();

    assert_eq!(names(&workflows), vec!["CI"]);
}

#[test]
fn default_ignores_directories_outside_the_standard_workflow_layout() {
    let tmp = git_repository_dir();
    write_workflow_in(tmp.path(), "custom/workflows", "ci.yml", "name: CI\n");

    let workflows = source().list_workflows(&repository(tmp.path())).unwrap();

    assert!(workflows.is_empty());
}
