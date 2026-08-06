use std::fs;

use ephemeral_act::{
    core::{
        ActRunConfig, Repository,
        ports::outbound::ActExecutor,
        value_objects::{
            ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, ContainerEngine, RepoPath,
            RepositoryName, Secret,
        },
    },
    infrastructure::{
        act_wrappers::{
            CiPlatform, forgejo_act_wrapper::ForgejoActWrapper,
            github_act_wrapper::GitHubActWrapper,
        },
        actions_executor::ActionsExecutor,
    },
};

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
    ActRunConfig::new(ContainerEngine::Podman)
}

/// Returns `true` when `act` is on `PATH` so integration tests
/// that shell out to it can run.  In CI containers the binary is absent and
/// these tests are skipped rather than failed.
fn act_available() -> bool {
    std::process::Command::new("which")
        .arg("act")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// CiPlatform::detect
// ---------------------------------------------------------------------------

#[test]
fn detects_github_when_no_forgejo_dir_exists() {
    let (_tmp, repo) = setup_repo(true, false);
    assert_eq!(CiPlatform::detect(&repo).unwrap(), CiPlatform::GitHub);
}

#[test]
fn detects_forgejo_when_forgejo_dir_exists() {
    let (_tmp, repo) = setup_repo(false, true);
    assert_eq!(CiPlatform::detect(&repo).unwrap(), CiPlatform::Forgejo);
}

#[test]
fn forgejo_takes_priority_when_both_dirs_exist() {
    let (_tmp, repo) = setup_repo(true, true);
    assert_eq!(CiPlatform::detect(&repo).unwrap(), CiPlatform::Forgejo);
}

#[test]
fn detect_errors_when_neither_platform_dir_exists() {
    let (_tmp, repo) = setup_repo(false, false);
    let result = CiPlatform::detect(&repo);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// GitHubActWrapper::build_args — act CLI interface
// ---------------------------------------------------------------------------

/// Repo path is passed via `-C` flag.
#[test]
fn repo_path_uses_c_flag() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert_eq!(args[0], "-C");
    assert_eq!(args[1], repo.path().as_path().to_string_lossy());
}

/// Workflow name passed via `-W`.
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

/// Event is positional (no `-e` flag).  It appears last, before extra args.
#[test]
fn includes_event_as_positional_arg() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().with_event(ActEvent::new("push".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);
    assert!(!args.contains(&"-e".to_string()));
    assert!(args.contains(&"push".to_string()));
}

/// Inputs use `--input`.
#[test]
fn includes_inputs_with_input_flag() {
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

/// Extra args are passed directly to `act` (no `--` separator).
#[test]
fn passes_extra_args_directly() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config()
        .add_extra_arg(ActExtraArg::new("--verbose".into()))
        .add_extra_arg(ActExtraArg::new("--dryrun".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);

    assert!(args.contains(&"--verbose".to_string()));
    assert!(args.contains(&"--dryrun".to_string()));
}

/// `--rm` and `--bind` are always emitted.
#[test]
fn emits_rm_and_bind_flags() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(args.contains(&"--rm".to_string()));
    assert!(args.contains(&"--bind".to_string()));
}

// ---------------------------------------------------------------------------
// ForgejoActWrapper::build_args — act CLI interface
// ---------------------------------------------------------------------------

/// Repo path is passed via `-C` flag.
#[test]
fn forgejo_repo_path_uses_c_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert_eq!(args[0], "-C");
    assert_eq!(args[1], repo.path().as_path().to_string_lossy());
}

/// `--workflows .forgejo/workflows/` is a direct act flag.
#[test]
fn includes_workflows_directory_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);

    let wf_pos = args.iter().position(|a| a == "--workflows").unwrap();
    assert_eq!(args[wf_pos + 1], ".forgejo/workflows/");
}

/// Workflow name passed via `-W`.
#[test]
fn forgejo_includes_workflow_flag_when_set() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_workflow(ActWorkflow::new("ci.yaml".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let wf_pos = args.iter().position(|a| a == "-W").unwrap();
    assert_eq!(args[wf_pos + 1], "ci.yaml");
}

#[test]
fn forgejo_omits_workflow_flag_when_none() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-W".to_string()));
}

#[test]
fn forgejo_includes_job_flag_when_set() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_job(ActJob::new("build".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let j_pos = args.iter().position(|a| a == "-j").unwrap();
    assert_eq!(args[j_pos + 1], "build");
}

#[test]
fn forgejo_omits_job_flag_when_none() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-j".to_string()));
}

/// Event is positional (no `-e` flag).
#[test]
fn forgejo_includes_event_as_positional_arg() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_event(ActEvent::new("push".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    assert!(!args.contains(&"-e".to_string()));
    assert!(args.contains(&"push".to_string()));
}

/// Inputs use `--input`.
#[test]
fn forgejo_includes_inputs_with_input_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().add_input(ActInput::new("debug".into(), "true".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let input_pos = args.iter().position(|a| a == "--input").unwrap();
    assert_eq!(args[input_pos + 1], "debug=true");
}

#[test]
fn forgejo_includes_secrets_with_s_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().add_secret(Secret::new("GITHUB_TOKEN=xxx".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let s_pos = args.iter().position(|a| a == "-s").unwrap();
    assert_eq!(args[s_pos + 1], "GITHUB_TOKEN=xxx");
}

/// Extra args are passed directly to `act` (no `--` separator).
#[test]
fn forgejo_passes_extra_args_directly() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config()
        .add_extra_arg(ActExtraArg::new("--verbose".into()))
        .add_extra_arg(ActExtraArg::new("--dryrun".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);

    assert!(args.contains(&"--verbose".to_string()));
    assert!(args.contains(&"--dryrun".to_string()));
}

/// `--rm` and `--bind` are always emitted.
#[test]
fn forgejo_emits_rm_and_bind_flags() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(args.contains(&"--rm".to_string()));
    assert!(args.contains(&"--bind".to_string()));
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
// GitHubActWrapper::execute_act (act is installed)
// ---------------------------------------------------------------------------

#[test]
fn github_execute_act_runs_act() {
    if !act_available() {
        return;
    }
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config();
    let result = GitHubActWrapper.execute_act(&config, &repo);
    // act is available — returns Ok even on no workflows
    assert!(
        result.is_ok(),
        "expected Ok when act is installed; got {:?}",
        result.err()
    );
    let execution = result.unwrap();
    assert!(
        !execution.success,
        "no workflows found, should report failure"
    );
    assert!(!execution.stderr.is_empty());
}

// ---------------------------------------------------------------------------
// ForgejoActWrapper::execute_act (act is installed)
// ---------------------------------------------------------------------------

#[test]
fn forgejo_execute_act_runs_act() {
    if !act_available() {
        return;
    }
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config();
    let result = ForgejoActWrapper.execute_act(&config, &repo);
    // act is available — returns Ok even on no workflows
    assert!(
        result.is_ok(),
        "expected Ok when act is installed; got {:?}",
        result.err()
    );
    let execution = result.unwrap();
    // No workflow files in .forgejo/workflows/ → act reports failure
    assert!(!execution.success);
}

// ---------------------------------------------------------------------------
// ActionsExecutor::execute_act dispatch
// ---------------------------------------------------------------------------

#[test]
fn actions_executor_dispatches_to_github_on_github_repo() {
    if !act_available() {
        return;
    }
    let (_tmp, repo) = setup_repo(true, false);
    let executor = ActionsExecutor::new();
    let config = test_config();
    let result = executor.execute_act(&config, &repo);
    // act is available — dispatch succeeds but reports failure (no workflows)
    assert!(result.is_ok());
    let execution = result.unwrap();
    assert!(!execution.success);
}

#[test]
fn actions_executor_dispatches_to_forgejo_on_forgejo_repo() {
    if !act_available() {
        return;
    }
    let (_tmp, repo) = setup_repo(false, true);
    let executor = ActionsExecutor::new();
    let config = test_config();
    let result = executor.execute_act(&config, &repo);
    // act is available for both platforms now
    assert!(result.is_ok());
}

#[test]
fn actions_executor_dispatches_to_forgejo_when_both_dirs_present() {
    if !act_available() {
        return;
    }
    let (_tmp, repo) = setup_repo(true, true);
    let executor = ActionsExecutor::new();
    let config = test_config();
    let result = executor.execute_act(&config, &repo);
    // Both use act — dispatch succeeds
    assert!(result.is_ok());
}
