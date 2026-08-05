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
// GitHubActWrapper::build_args — act-ephemeral.sh interface
// ---------------------------------------------------------------------------

/// Repo path is the first positional argument (no `-C` flag).
#[test]
fn repo_path_is_first_positional_arg() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert_eq!(args[0], repo.path().as_path().to_string_lossy());
    assert!(!args.contains(&"-C".to_string()));
}

/// Container engine passed via `-c`.
#[test]
fn includes_container_engine_flag() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    let c_pos = args.iter().position(|a| a == "-c").unwrap();
    assert_eq!(args[c_pos + 1], "podman");
}

/// Workflow name passed via `-w` (was `-W`).
#[test]
fn includes_workflow_flag_when_set() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().with_workflow(ActWorkflow::new("ci.yaml".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);
    let wf_pos = args.iter().position(|a| a == "-w").unwrap();
    assert_eq!(args[wf_pos + 1], "ci.yaml");
}

#[test]
fn omits_workflow_flag_when_none() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-w".to_string()));
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

/// Event is now a flagged arg (`-e push`) rather than positional.
#[test]
fn includes_event_as_flagged_arg() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config().with_event(ActEvent::new("push".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);
    let e_pos = args.iter().position(|a| a == "-e").unwrap();
    assert_eq!(args[e_pos + 1], "push");
}

#[test]
fn omits_event_flag_when_none() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-e".to_string()));
}

/// Inputs use `-i` (was `--input`).
#[test]
fn includes_inputs_with_i_flag() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config()
        .add_input(ActInput::new("debug".into(), "true".into()))
        .add_input(ActInput::new("target".into(), "x86_64".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);

    let first_pos = args.iter().position(|a| a == "-i").unwrap();
    assert_eq!(args[first_pos + 1], "debug=true");

    let second_pos = args[first_pos + 2..]
        .iter()
        .position(|a| a == "-i")
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

/// Extra args are passed after `--`.
#[test]
fn passes_extra_args_after_double_dash() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config()
        .add_extra_arg(ActExtraArg::new("--verbose".into()))
        .add_extra_arg(ActExtraArg::new("--dryrun".into()));
    let args = GitHubActWrapper::build_args(&config, &repo);

    let dash_pos = args.iter().position(|a| a == "--").unwrap();
    assert_eq!(args[dash_pos + 1], "--verbose");
    assert_eq!(args[dash_pos + 2], "--dryrun");
}

/// No `--` separator when there are no extra args.
#[test]
fn no_double_dash_when_no_extra_args() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"--".to_string()));
}

/// Script handles `--rm` and `--bind` internally — wrapper never emits them.
#[test]
fn does_not_emit_rm_or_bind_flags() {
    let (_tmp, repo) = setup_repo(true, false);
    let args = GitHubActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"--rm".to_string()));
    assert!(!args.contains(&"--bind".to_string()));
}

// ---------------------------------------------------------------------------
// ForgejoActWrapper::build_args — act-ephemeral.sh interface
// ---------------------------------------------------------------------------

/// Repo path is the first positional argument (no `-C` flag).
#[test]
fn forgejo_repo_path_is_first_positional_arg() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert_eq!(args[0], repo.path().as_path().to_string_lossy());
    assert!(!args.contains(&"-C".to_string()));
}

/// Container engine passed via `-c`.
#[test]
fn forgejo_includes_container_engine_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    let c_pos = args.iter().position(|a| a == "-c").unwrap();
    assert_eq!(args[c_pos + 1], "podman");
}

/// `--workflows .forgejo/workflows/` appears after the `--` separator.
#[test]
fn includes_workflows_directory_after_double_dash() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);

    let dash_pos = args.iter().position(|a| a == "--").unwrap();
    let wf_pos = args.iter().position(|a| a == "--workflows").unwrap();
    assert!(wf_pos > dash_pos, "--workflows must come after --");
    assert_eq!(args[wf_pos + 1], ".forgejo/workflows/");
}

#[test]
fn forgejo_includes_workflow_flag_when_set() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_workflow(ActWorkflow::new("ci.yaml".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let wf_pos = args.iter().position(|a| a == "-w").unwrap();
    assert_eq!(args[wf_pos + 1], "ci.yaml");
}

#[test]
fn forgejo_omits_workflow_flag_when_none() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-w".to_string()));
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

#[test]
fn forgejo_includes_event_as_flagged_arg() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().with_event(ActEvent::new("push".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let e_pos = args.iter().position(|a| a == "-e").unwrap();
    assert_eq!(args[e_pos + 1], "push");
}

#[test]
fn forgejo_omits_event_flag_when_none() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"-e".to_string()));
}

#[test]
fn forgejo_includes_inputs_with_i_flag() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config().add_input(ActInput::new("debug".into(), "true".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);
    let input_pos = args.iter().position(|a| a == "-i").unwrap();
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

/// Extra args appear after `--workflows .forgejo/workflows/`.
#[test]
fn forgejo_passes_extra_args_after_workflows() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config()
        .add_extra_arg(ActExtraArg::new("--verbose".into()))
        .add_extra_arg(ActExtraArg::new("--dryrun".into()));
    let args = ForgejoActWrapper::build_args(&config, &repo);

    let wf_pos = args.iter().position(|a| a == "--workflows").unwrap();
    let verbose_pos = args.iter().position(|a| a == "--verbose").unwrap();
    let dryrun_pos = args.iter().position(|a| a == "--dryrun").unwrap();
    assert!(
        verbose_pos > wf_pos,
        "extra args must come after --workflows"
    );
    assert!(
        dryrun_pos > wf_pos,
        "extra args must come after --workflows"
    );
}

/// Script handles `--rm` and `--bind` internally — wrapper never emits them.
#[test]
fn forgejo_does_not_emit_rm_or_bind_flags() {
    let (_tmp, repo) = setup_repo(false, true);
    let args = ForgejoActWrapper::build_args(&test_config(), &repo);
    assert!(!args.contains(&"--rm".to_string()));
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
// GitHubActWrapper::execute_act (act-ephemeral.sh is installed)
// ---------------------------------------------------------------------------

#[test]
fn github_execute_act_runs_ephemeral_script() {
    let (_tmp, repo) = setup_repo(true, false);
    let config = test_config();
    let result = GitHubActWrapper.execute_act(&config, &repo);
    // act-ephemeral.sh is available — returns Ok even on no workflows
    assert!(
        result.is_ok(),
        "expected Ok when act-ephemeral.sh is installed; got {:?}",
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
// ForgejoActWrapper::execute_act (act-ephemeral.sh is installed)
// ---------------------------------------------------------------------------

#[test]
fn forgejo_execute_act_runs_ephemeral_script() {
    let (_tmp, repo) = setup_repo(false, true);
    let config = test_config();
    let result = ForgejoActWrapper.execute_act(&config, &repo);
    // act-ephemeral.sh is available — returns Ok even on no workflows
    assert!(
        result.is_ok(),
        "expected Ok when act-ephemeral.sh is installed; got {:?}",
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
    let (_tmp, repo) = setup_repo(true, false);
    let executor = ActionsExecutor::new();
    let config = test_config();
    let result = executor.execute_act(&config, &repo);
    // act-ephemeral.sh is available — dispatch succeeds but reports failure (no workflows)
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
    // act-ephemeral.sh is available for both platforms now
    assert!(result.is_ok());
}

#[test]
fn actions_executor_dispatches_to_forgejo_when_both_dirs_present() {
    let (_tmp, repo) = setup_repo(true, true);
    let executor = ActionsExecutor::new();
    let config = test_config();
    let result = executor.execute_act(&config, &repo);
    // Both use act-ephemeral.sh — dispatch succeeds
    assert!(result.is_ok());
}
