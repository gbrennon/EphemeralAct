use ephemeral_act::core::ports::outbound::ActExecutor;
use ephemeral_act::infrastructure::actions_executor::ActionsExecutor;


use std::fs;

use ephemeral_act::core::value_objects::{
    ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, ContainerDaemonSocket, ContainerEngine,
    RepoPath, RepositoryName, Secret,
};
use ephemeral_act::core::{ActRunConfig, Repository};
use ephemeral_act::infrastructure::act_wrappers::CiPlatform;
use ephemeral_act::infrastructure::act_wrappers::forgejo_act_wrapper::ForgejoActWrapper;
use ephemeral_act::infrastructure::act_wrappers::github_act_wrapper::GitHubActWrapper;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a temp directory with a `.git/` subdir (required by `RepoPath`),
/// optionally creating `.github/workflows/` and/or `.forgejo/workflows/`.
fn setup_repo(create_github: bool, create_forgejo: bool) -> (tempfile::TempDir, Repository) {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".git")).unwrap();
    if create_github {
        fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    }
    if create_forgejo {
        fs::create_dir_all(tmp.path().join(".forgejo/workflows")).unwrap();
    }
    let repo = Repository::new(
        RepoPath::new(tmp.path().to_path_buf()).unwrap(),
        RepositoryName::new("test-repo".into()).unwrap(),
    );
    (tmp, repo)
}

fn test_config() -> ActRunConfig {
    ActRunConfig::new(
        ContainerEngine::Podman,
        ContainerDaemonSocket::new("unix:///run/podman/podman.sock".into()),
    )
}

// ---------------------------------------------------------------------------
// CiPlatform::detect
// ---------------------------------------------------------------------------

#[test]
fn detects_github_when_no_forgejo_dir_exists() {
    let (_tmp, repo) = setup_repo(true, false);
    let platform = CiPlatform::detect(&repo);
    assert!(matches!(platform, CiPlatform::GitHub));
}

#[test]
fn detects_forgejo_when_forgejo_dir_exists() {
    let (_tmp, repo) = setup_repo(false, true);
    let platform = CiPlatform::detect(&repo);
    assert!(matches!(platform, CiPlatform::Forgejo));
}

#[test]
fn forgejo_takes_priority_when_both_dirs_exist() {
    let (_tmp, repo) = setup_repo(true, true);
    let platform = CiPlatform::detect(&repo);
    assert!(matches!(platform, CiPlatform::Forgejo));
}

// ---------------------------------------------------------------------------
// GitHubActWrapper::build_args
// ---------------------------------------------------------------------------
#[test]
fn includes_working_directory_with_c_flag() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    let c_pos = args.iter().position(|a| a == "-C").unwrap();
    assert_eq!(args[c_pos + 1], repo.path().as_path().to_string_lossy());
}

#[test]
fn includes_container_daemon_socket() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    let socket_pos = args
        .iter()
        .position(|a| a == "--container-daemon-socket")
        .unwrap();
    assert_eq!(args[socket_pos + 1], "unix:///run/podman/podman.sock");
}

#[test]
fn includes_workflow_flag_when_set() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().with_workflow(ActWorkflow::new("ci.yaml".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);
    let wf_pos = args.iter().position(|a| a == "-W").unwrap();
    assert_eq!(args[wf_pos + 1], "ci.yaml");
}

#[test]
fn omits_workflow_flag_when_none() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-W".to_string()));
}

#[test]
fn includes_job_flag_when_set() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().with_job(ActJob::new("build".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);
    let j_pos = args.iter().position(|a| a == "-j").unwrap();
    assert_eq!(args[j_pos + 1], "build");
}

#[test]
fn omits_job_flag_when_none() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-j".to_string()));
}

#[test]
fn includes_event_arg() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().with_event(ActEvent::new("push".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);
    assert!(args.contains(&"push".to_string()));
}

#[test]
fn includes_inputs_with_key_equals_value() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config()
        .add_input(ActInput::new("debug".into(), "true".into()))
        .add_input(ActInput::new("target".into(), "x86_64".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);

    let first_pos = args.iter().position(|a| a == "--input").unwrap();
    assert_eq!(args[first_pos + 1], "debug=true");

    let second_pos = args[first_pos + 2..]
        .iter()
        .position(|a| a == "--input")
        .unwrap()
        + first_pos
        + 2;
    assert_eq!(args[second_pos + 1], "target=x86_64");
}

#[test]
fn includes_secrets_with_s_flag() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().add_secret(Secret::new("GITHUB_TOKEN=xxx".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);
    let s_pos = args.iter().position(|a| a == "-s").unwrap();
    assert_eq!(args[s_pos + 1], "GITHUB_TOKEN=xxx");
}

#[test]
fn passes_extra_args_through_directly() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config()
        .add_extra_arg(ActExtraArg::new("--verbose".into()))
        .add_extra_arg(ActExtraArg::new("--dryrun".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);
    assert!(args.contains(&"--verbose".to_string()));
    assert!(args.contains(&"--dryrun".to_string()));
}

#[test]
fn includes_rm_flag_by_default() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(args.contains(&"--rm".to_string()));
}

#[test]
fn omits_rm_flag_when_false() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().with_rm(false);
    let args = GitHubActWrapper::build_args(&config, &repo);
    assert!(!args.contains(&"--rm".to_string()));
}

#[test]
fn includes_bind_flag_by_default() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(args.contains(&"--bind".to_string()));
}

#[test]
fn omits_bind_flag_when_false() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().with_bind(false);
    let args = GitHubActWrapper::build_args(&config, &repo);
    assert!(!args.contains(&"--bind".to_string()));
}

#[test]
fn event_comes_before_extra_args() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config()
        .with_event(ActEvent::new("push".into()))
        .add_extra_arg(ActExtraArg::new("--verbose".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);

    let event_pos = args.iter().position(|a| a == "push").unwrap();
    let verbose_pos = args.iter().position(|a| a == "--verbose").unwrap();
    assert!(event_pos < verbose_pos);
}

// ---------------------------------------------------------------------------
// ForgejoActWrapper::build_args
// ---------------------------------------------------------------------------

#[test]
fn includes_workflows_directory_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    let wf_pos = args.iter().position(|a| a == "--workflows").unwrap();
    assert_eq!(args[wf_pos + 1], ".forgejo/workflows/");
}

#[test]
fn includes_forgejo_working_directory_with_c_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    let c_pos = args.iter().position(|a| a == "-C").unwrap();
    assert_eq!(args[c_pos + 1], repo.path().as_path().to_string_lossy());
}

#[test]
fn includes_forgejo_container_daemon_socket() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    let socket_pos = args
        .iter()
        .position(|a| a == "--container-daemon-socket")
        .unwrap();
    assert_eq!(args[socket_pos + 1], "unix:///run/podman/podman.sock");
}

#[test]
fn includes_forgejo_workflow_flag_when_set() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_workflow(ActWorkflow::new("ci.yaml".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let wf_pos = args.iter().position(|a| a == "-W").unwrap();
    assert_eq!(args[wf_pos + 1], "ci.yaml");
}

#[test]
fn omits_forgejo_workflow_flag_when_none() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-W".to_string()));
}

#[test]
fn includes_forgejo_job_flag_when_set() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_job(ActJob::new("build".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let j_pos = args.iter().position(|a| a == "-j").unwrap();
    assert_eq!(args[j_pos + 1], "build");
}

#[test]
fn omits_forgejo_job_flag_when_none() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-j".to_string()));
}

#[test]
fn includes_forgejo_event_arg() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_event(ActEvent::new("push".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    assert!(args.contains(&"push".to_string()));
}

#[test]
fn uses_input_flag_for_forgejo() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().add_input(ActInput::new("debug".into(), "true".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let input_pos = args.iter().position(|a| a == "--input").unwrap();
    assert_eq!(args[input_pos + 1], "debug=true");
}

#[test]
fn includes_forgejo_secrets_with_s_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().add_secret(Secret::new("GITHUB_TOKEN=xxx".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let s_pos = args.iter().position(|a| a == "-s").unwrap();
    assert_eq!(args[s_pos + 1], "GITHUB_TOKEN=xxx");
}

#[test]
fn passes_forgejo_extra_args_through_directly() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config()
        .add_extra_arg(ActExtraArg::new("--verbose".into()))
        .add_extra_arg(ActExtraArg::new("--dryrun".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    assert!(args.contains(&"--verbose".to_string()));
    assert!(args.contains(&"--dryrun".to_string()));
}

#[test]
fn includes_forgejo_rm_flag_by_default() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(args.contains(&"--rm".to_string()));
}

#[test]
fn omits_forgejo_rm_flag_when_false() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_rm(false);
    let args = ForgejoActWrapper::build_args(&config, &repo);
    assert!(!args.contains(&"--rm".to_string()));
}

#[test]
fn includes_forgejo_bind_flag_by_default() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(args.contains(&"--bind".to_string()));
}

#[test]
fn omits_forgejo_bind_flag_when_false() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_bind(false);
    let args = ForgejoActWrapper::build_args(&config, &repo);
    assert!(!args.contains(&"--bind".to_string()));
}

// ---------------------------------------------------------------------------
// ActionsExecutor construction
// ---------------------------------------------------------------------------

#[test]
fn actions_executor_new_creates_instance() {
    let _executor = ActionsExecutor::new();
}

#[test]
fn actions_executor_default_works() {
    let _executor: ActionsExecutor = Default::default();
}

// ---------------------------------------------------------------------------
// GitHubActWrapper::execute_act (act binary is installed)
// ---------------------------------------------------------------------------

#[test]
fn github_execute_act_runs_act_binary() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config();
    let result = GitHubActWrapper.execute_act(&config, &repo);
    // act binary is available — returns Ok(ExecutionResult) even on no workflows
    assert!(result.is_ok(), "expected Ok when act is installed; got {:?}", result.err());
    let execution = result.unwrap();
    assert!(!execution.success, "no workflows found, should report failure");
    assert!(!execution.stderr.is_empty());
}

// ---------------------------------------------------------------------------
// ForgejoActWrapper::execute_act error path
// ---------------------------------------------------------------------------

#[test]
fn forgejo_execute_act_errors_when_act_runner_binary_absent() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config();
    let result = ForgejoActWrapper.execute_act(&config, &repo);
    assert!(result.is_err(), "expected error when act_runner is not installed; got {:?}", result.ok());
}

// ---------------------------------------------------------------------------
// ActionsExecutor::execute_act dispatch
// ---------------------------------------------------------------------------

#[test]
fn actions_executor_dispatches_to_github_on_github_repo() {
    let (_tmp, repo) = setup_repo(true, false);
    let executor = ActionsExecutor::new();
    let config = test_config();
    let result = executor.execute_act(&config, &repo);
    // act binary is available — dispatch succeeds but reports failure (no workflows)
    assert!(result.is_ok());
    let execution = result.unwrap();
    assert!(!execution.success);
}

#[test]
fn actions_executor_dispatches_to_forgejo_on_forgejo_repo() {
    let (_tmp, repo) = setup_repo(false, true);
    let executor = ActionsExecutor::new();
    let config = test_config();
    let result = executor.execute_act(&config, &repo);
    assert!(result.is_err());
}

#[test]
fn actions_executor_dispatches_to_forgejo_when_both_dirs_present() {
    let (_tmp, repo) = setup_repo(true, true);
    let executor = ActionsExecutor::new();
    let config = test_config();
    let result = executor.execute_act(&config, &repo);
    assert!(result.is_err());
}
