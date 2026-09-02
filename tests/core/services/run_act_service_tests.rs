#[cfg(test)]
mod tests {
    use std::{path::Path, rc::Rc};

    use ephemeral_act::core::{
        ActRunConfig, ActWorkflow, RepoPath, Repository, RepositoryName,
        dtos::{RunActRequest, StepType},
        events::DomainEvent,
        ports::{inbound::run_act_port::RunActPort, outbound::ExecResult},
        services::{
            execute_action_service::ExecuteActionService,
            run_act_service::{ALL_WORKFLOWS_SUMMARY_NAME, RunActService},
        },
        value_objects::{ActEvent, ActInput, Secret},
    };

    use crate::common::fakes::{
        fake_action_fetcher::FakeActionFetcher, fake_event_publisher::FakeEventPublisher,
        fake_image_mapper::FakeImageMapper, fake_runtime::FakeRuntime,
        shared_fake_runtime::SharedFakeRuntime,
    };

    fn make_repo(path: &Path) -> Repository {
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            std::fs::create_dir_all(&git_dir).ok();
        }
        let repo_path = RepoPath::new(path.to_path_buf()).unwrap();
        let name = RepositoryName::new("test-repo".into()).unwrap();
        Repository::new(repo_path, name)
    }

    fn write_workflow(dir: &Path, name: &str, body: &str) {
        std::fs::create_dir_all(dir.join(".forgejo/workflows")).unwrap();
        std::fs::write(dir.join(".forgejo/workflows").join(name), body).unwrap();
    }

    fn write_github_workflow(dir: &Path, name: &str, body: &str) {
        std::fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        std::fs::write(dir.join(".github/workflows").join(name), body).unwrap();
    }

    fn write_action(dir: &Path, relative: &str, body: &str) {
        let action_dir = dir.join(relative);
        std::fs::create_dir_all(&action_dir).unwrap();
        std::fs::write(action_dir.join("action.yml"), body).unwrap();
    }

    fn push_result(runtime: &FakeRuntime, exit_code: i64, stdout: &str, stderr: &str) {
        runtime.exec_results.borrow_mut().push(ExecResult {
            exit_code,
            stdout: stdout.into(),
            stderr: stderr.into(),
        });
    }

    /// Publisher wired to the action executor, mirroring the production bus.
    fn publisher_with_action_executor(mirror_dir: &Path) -> FakeEventPublisher {
        FakeEventPublisher::with_action_handler(Rc::new(ExecuteActionService::new(
            FakeActionFetcher::returning(mirror_dir.to_path_buf()),
        )))
    }

    #[test]
    fn execute_executes_workflow_and_publishes_completed_event() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "build.yml",
            "name: Event\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "hi\n", "");
        let publisher = FakeEventPublisher::new();
        let service = RunActService::new(runtime, FakeImageMapper, publisher.clone());

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        assert!(result.success);
        assert!(matches!(
            publisher.events().as_slice(),
            [DomainEvent::ActRunCompleted(_)]
        ));
    }

    #[test]
    fn execute_finds_workflow_in_forgejo_dir() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "ci.yml",
            "name: Ci\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "hi\n", "");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config =
            ActRunConfig::new().with_workflow(ActWorkflow::new(".forgejo/workflows/ci.yml".into()));

        let result = service.execute(RunActRequest::new(config, repo)).unwrap();

        assert!(result.success);
    }

    #[test]
    fn execute_errors_on_nonexistent_workflow() {
        let repo = make_repo(Path::new(env!("CARGO_MANIFEST_DIR")));
        let service = RunActService::new(
            FakeRuntime::new(),
            FakeImageMapper,
            FakeEventPublisher::new(),
        );
        let config = ActRunConfig::new().with_workflow(ActWorkflow::new("nonexistent.yml".into()));

        let err = service
            .execute(RunActRequest::new(config, repo))
            .unwrap_err();

        assert!(err.to_string().contains("nonexistent.yml"), "{}", err);
    }

    #[test]
    fn execute_errors_when_no_workflow_found() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = make_repo(tmp.path());
        let service = RunActService::new(
            FakeRuntime::new(),
            FakeImageMapper,
            FakeEventPublisher::new(),
        );

        let err = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap_err();

        assert!(err.to_string().contains("workflows directory"), "{}", err);
    }

    #[test]
    fn execute_reports_failure_on_step_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "fail.yml",
            "name: Fail\non: push\njobs:\n  fail:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 1, "", "fail");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        assert!(!result.success);
    }

    #[test]
    fn execute_continues_run_when_failing_step_has_continue_on_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "continue.yml",
            "name: Continue\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n        continue-on-error: true\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 1, "", "fail");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        assert!(result.success, "run with continue-on-error should succeed");
        assert!(result.job_summaries[0].success);
    }

    #[test]
    fn execute_records_failed_step_details_when_continue_on_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "continue.yml",
            "name: Continue\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n        continue-on-error: true\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 1, "", "fail");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        let step = &result.job_summaries[0].steps[0];
        assert_eq!(step.exit_code, Some(1));
        assert!(step.continue_on_error);
    }

    #[test]
    fn execute_resolves_secret_expressions_in_run_scripts() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "publish.yml",
            "name: Publish\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo publish --token ${{ secrets.CRATES_IO_STAGING_TOKEN }}\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = SharedFakeRuntime::new();
        let service =
            RunActService::new(runtime.clone(), FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new().add_secret(Secret::new(
            "CRATES_IO_STAGING_TOKEN".into(),
            "test-staging-token".into(),
        ));

        service.execute(RunActRequest::new(config, repo)).unwrap();

        assert_eq!(
            runtime.executed_scripts(),
            vec!["cargo publish --token test-staging-token".to_string()]
        );
    }

    #[test]
    fn execute_resolves_input_and_event_expressions_in_run_scripts() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "publish.yml",
            "name: Publish\non: workflow_dispatch\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - run: deploy ${{ inputs.mode }} ${{ github.event_name }}\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = SharedFakeRuntime::new();
        let service =
            RunActService::new(runtime.clone(), FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new()
            .add_input(ActInput::new("mode".into(), "staging".into()))
            .with_event(ActEvent::new("pull_request".into()));

        service.execute(RunActRequest::new(config, repo)).unwrap();

        assert_eq!(
            runtime.executed_scripts(),
            vec!["deploy staging pull_request".to_string()]
        );
    }

    #[test]
    fn execute_runs_local_composite_action_through_the_event_handler() {
        let tmp = tempfile::tempdir().unwrap();
        write_action(
            tmp.path(),
            ".forgejo/actions/my-action",
            "name: My Action\nruns:\n  using: composite\n  steps:\n    - run: echo hi\n",
        );
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.forgejo/actions/my-action\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "hi", "");
        let publisher = publisher_with_action_executor(tmp.path());
        let service = RunActService::new(runtime, FakeImageMapper, publisher.clone());

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        assert!(result.success);
        assert_eq!(
            result.job_summaries[0].steps[0].step_type,
            StepType::Composite
        );
        assert!(
            publisher
                .events()
                .iter()
                .any(|event| matches!(event, DomainEvent::ActionExecutionRequested(_))),
            "the runner should request action execution through an event"
        );
    }

    #[test]
    fn execute_resolves_secrets_inside_composite_action_scripts() {
        let tmp = tempfile::tempdir().unwrap();
        write_action(
            tmp.path(),
            ".forgejo/actions/publish",
            "name: Publish\nruns:\n  using: composite\n  steps:\n    - run: cargo publish --token ${{ secrets.CRATES_IO_STAGING_TOKEN }}\n      shell: bash\n",
        );
        write_workflow(
            tmp.path(),
            "publish.yml",
            "name: Publish\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.forgejo/actions/publish\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = SharedFakeRuntime::new();
        let service = RunActService::new(
            runtime.clone(),
            FakeImageMapper,
            publisher_with_action_executor(tmp.path()),
        );
        let config = ActRunConfig::new().add_secret(Secret::new(
            "CRATES_IO_STAGING_TOKEN".into(),
            "test-staging-token".into(),
        ));

        let result = service.execute(RunActRequest::new(config, repo)).unwrap();

        assert!(result.success);
        assert_eq!(
            runtime.executed_scripts(),
            vec!["cargo publish --token test-staging-token".to_string()]
        );
    }

    #[test]
    fn execute_runs_remote_action_fetched_from_any_forge() {
        let tmp = tempfile::tempdir().unwrap();
        let mirror = tempfile::tempdir().unwrap();
        std::fs::write(
            mirror.path().join("action.yml"),
            "name: Cache\nruns:\n  using: composite\n  steps:\n    - run: echo cached\n      shell: bash\n",
        )
        .unwrap();
        write_workflow(
            tmp.path(),
            "cache.yml",
            "name: Cache\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: https://data.forgejo.org/actions/cache@v4\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "cached\n", "");
        let service = RunActService::new(
            runtime,
            FakeImageMapper,
            publisher_with_action_executor(mirror.path()),
        );

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        let step = &result.job_summaries[0].steps[0];
        assert!(result.success, "{}", step.stderr);
        assert_eq!(step.step_type, StepType::Uses);
        assert_eq!(step.stdout, "cached\n");
    }

    #[test]
    fn execute_fails_action_step_when_no_handler_runs_it() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "cache.yml",
            "name: Cache\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: https://data.forgejo.org/actions/cache@v4\n",
        );
        let repo = make_repo(tmp.path());
        let service = RunActService::new(
            FakeRuntime::new(),
            FakeImageMapper,
            FakeEventPublisher::new(),
        );

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        assert!(!result.success);
        assert!(
            result.job_summaries[0].steps[0]
                .stderr
                .contains("no handler executed the action"),
            "{}",
            result.job_summaries[0].steps[0].stderr
        );
    }

    #[test]
    fn execute_preserves_partial_stdout_on_action_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_action(
            tmp.path(),
            ".forgejo/actions/broken",
            "name: Broken\nruns:\n  using: composite\n  steps:\n    - run: echo partial-output\n    - name: not a runnable step\n",
        );
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.forgejo/actions/broken\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "partial-output\n", "");
        let service = RunActService::new(
            runtime,
            FakeImageMapper,
            publisher_with_action_executor(tmp.path()),
        );

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        let step = &result.job_summaries[0].steps[0];
        assert_eq!(step.stdout, "partial-output\n");
        assert_eq!(step.exit_code, None);
        assert!(step.stderr.contains("step error:"), "{}", step.stderr);
    }

    #[test]
    fn execute_fails_the_step_for_container_actions() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: docker://node:20\n",
        );
        let repo = make_repo(tmp.path());
        let service = RunActService::new(
            FakeRuntime::new(),
            FakeImageMapper,
            publisher_with_action_executor(tmp.path()),
        );

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        assert!(!result.success);
        let step = &result.job_summaries[0].steps[0];
        assert_eq!(step.step_type, StepType::Uses);
        assert!(
            step.stderr.contains("unsupported action"),
            "{}",
            step.stderr
        );
    }

    #[test]
    fn execute_skips_checkout_actions_because_the_workspace_is_mounted() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "ci.yml",
            "name: Ci\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n",
        );
        let repo = make_repo(tmp.path());
        let service = RunActService::new(
            FakeRuntime::new(),
            FakeImageMapper,
            publisher_with_action_executor(tmp.path()),
        );

        let result = service
            .execute(RunActRequest::new(ActRunConfig::new(), repo))
            .unwrap();

        assert!(result.success);
        assert!(
            result.job_summaries[0].steps[0]
                .stdout
                .contains("[skipped]"),
            "{}",
            result.job_summaries[0].steps[0].stdout
        );
    }

    #[test]
    fn execute_all_workflows_summarizes_jobs_from_every_workflow_file() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "alpha.yml",
            "name: Alpha\non: push\njobs:\n  first:\n    name: One\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
        );
        write_github_workflow(
            tmp.path(),
            "beta.yml",
            "name: Beta\non: push\njobs:\n  second:\n    name: Two\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "hi\n", "");
        push_result(&runtime, 0, "hi\n", "");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new().with_all_workflows(true);

        let result = service.execute(RunActRequest::new(config, repo)).unwrap();

        assert_eq!(result.name, ALL_WORKFLOWS_SUMMARY_NAME);
        assert!(result.success);
        assert_eq!(
            result
                .job_summaries
                .iter()
                .map(|job| job.name.clone())
                .collect::<Vec<_>>(),
            vec![
                Some("Alpha / One".to_string()),
                Some("Beta / Two".to_string())
            ]
        );
    }

    #[test]
    fn execute_all_workflows_errors_when_repository_has_no_workflow_files() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = make_repo(tmp.path());
        let service = RunActService::new(
            FakeRuntime::new(),
            FakeImageMapper,
            FakeEventPublisher::new(),
        );
        let config = ActRunConfig::new().with_all_workflows(true);

        let error = service
            .execute(RunActRequest::new(config, repo))
            .unwrap_err()
            .to_string();

        assert!(error.contains("no workflow files found"), "{}", error);
    }

    #[test]
    fn execute_all_workflows_fails_the_run_when_any_job_fails() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "alpha.yml",
            "name: Alpha\non: push\njobs:\n  first:\n    name: One\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
        );
        write_github_workflow(
            tmp.path(),
            "beta.yml",
            "name: Beta\non: push\njobs:\n  second:\n    name: Two\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n      - run: exit 1\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "", "");
        push_result(&runtime, 1, "", "boom");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new().with_all_workflows(true);

        let result = service.execute(RunActRequest::new(config, repo)).unwrap();

        assert!(!result.success);
        assert_eq!(
            result
                .job_summaries
                .iter()
                .map(|job| job.success)
                .collect::<Vec<_>>(),
            vec![true, false]
        );
    }

    #[test]
    fn execute_all_workflows_keeps_run_successful_when_erroring_step_continues_on_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_action(
            tmp.path(),
            ".forgejo/actions/broken",
            "name: Broken\nruns:\n  using: composite\n  steps:\n    - run: echo partial-output\n    - name: not a runnable step\n",
        );
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.forgejo/actions/broken\n        continue-on-error: true\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "partial-output\n", "");
        let service = RunActService::new(
            runtime,
            FakeImageMapper,
            publisher_with_action_executor(tmp.path()),
        );
        let config = ActRunConfig::new().with_all_workflows(true);

        let result = service.execute(RunActRequest::new(config, repo)).unwrap();

        assert!(result.success);
        let step = &result.job_summaries[0].steps[0];
        assert_eq!(step.exit_code, None);
        assert_eq!(step.stdout, "partial-output\n");
        assert!(step.stderr.contains("step error:"), "{}", step.stderr);
    }

    #[test]
    fn execute_all_workflows_publishes_one_completed_event_with_every_container() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "alpha.yml",
            "name: Alpha\non: push\njobs:\n  first:\n    name: One\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
        );
        write_github_workflow(
            tmp.path(),
            "beta.yml",
            "name: Beta\non: push\njobs:\n  second:\n    name: Two\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
        );
        let repo = make_repo(tmp.path());
        let publisher = FakeEventPublisher::new();
        let service = RunActService::new(FakeRuntime::new(), FakeImageMapper, publisher.clone());
        let config = ActRunConfig::new().with_all_workflows(true);

        service.execute(RunActRequest::new(config, repo)).unwrap();

        let events = publisher.events();
        assert_eq!(events.len(), 1);
        let Some(DomainEvent::ActRunCompleted(payload)) = events.first() else {
            panic!("expected a single completed event, got {events:?}");
        };
        assert_eq!(payload.container_names.len(), 2);
        assert!(payload.success);
    }
}
