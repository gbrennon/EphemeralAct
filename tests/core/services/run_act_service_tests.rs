#[cfg(test)]
mod tests {
    use std::path::Path;

    use ephemeral_act::core::{
        ActRunConfig, ActWorkflow, RepoPath, Repository, RepositoryName,
        dtos::{RunActRequest, StepType},
        events::DomainEvent,
        ports::{inbound::run_act_port::RunActPort, outbound::ExecResult},
        services::run_act_service::RunActService,
    };

    use crate::common::fakes::{
        fake_event_publisher::FakeEventPublisher, fake_image_mapper::FakeImageMapper,
        fake_runtime::FakeRuntime,
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

    fn push_result(runtime: &FakeRuntime, exit_code: i64, stdout: &str, stderr: &str) {
        runtime.exec_results.borrow_mut().push(ExecResult {
            exit_code,
            stdout: stdout.into(),
            stderr: stderr.into(),
        });
    }

    #[test]
    fn execute_executes_workflow_and_publishes_event() {
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
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
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
        let runtime = FakeRuntime::new();
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
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
        let runtime = FakeRuntime::new();
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new();
        let err = service
            .execute(RunActRequest::new(config, repo))
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
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
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
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
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
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
        let step = &result.job_summaries[0].steps[0];
        assert_eq!(step.exit_code, Some(1));
        assert!(step.continue_on_error);
    }

    #[test]
    fn execute_labels_local_action_step_as_composite() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".forgejo/actions/my-action")).unwrap();
        std::fs::write(
            tmp.path().join(".forgejo/actions/my-action/action.yml"),
            "name: My Action\nruns:\n  using: composite\n  steps:\n    - run: echo hi\n",
        )
        .unwrap();
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.forgejo/actions/my-action\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "hi", "");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
        assert!(result.success);
        assert_eq!(
            result.job_summaries[0].steps[0].step_type,
            StepType::Composite
        );
    }

    #[test]
    fn execute_preserves_partial_stdout_on_step_error() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".forgejo/actions/broken")).unwrap();
        std::fs::write(
            tmp.path().join(".forgejo/actions/broken/action.yml"),
            "name: Broken\nruns:\n  using: composite\n  steps:\n    - run: echo partial-output\n    - name: not a runnable step\n",
        )
        .unwrap();
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.forgejo/actions/broken\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "partial-output\n", "");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
        let step = &result.job_summaries[0].steps[0];
        assert_eq!(step.stdout, "partial-output\n");
    }

    #[test]
    fn execute_marks_failed_step_without_exit_code() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".forgejo/actions/broken")).unwrap();
        std::fs::write(
            tmp.path().join(".forgejo/actions/broken/action.yml"),
            "name: Broken\nruns:\n  using: composite\n  steps:\n    - run: echo partial-output\n    - name: not a runnable step\n",
        )
        .unwrap();
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.forgejo/actions/broken\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "partial-output\n", "");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
        let step = &result.job_summaries[0].steps[0];
        assert_eq!(step.exit_code, None);
    }

    #[test]
    fn execute_annotates_step_stderr_on_error() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".forgejo/actions/broken")).unwrap();
        std::fs::write(
            tmp.path().join(".forgejo/actions/broken/action.yml"),
            "name: Broken\nruns:\n  using: composite\n  steps:\n    - run: echo partial-output\n    - name: not a runnable step\n",
        )
        .unwrap();
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.forgejo/actions/broken\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "partial-output\n", "");
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
        let step = &result.job_summaries[0].steps[0];
        assert!(step.stderr.contains("step error:"), "{}", step.stderr);
    }

    #[test]
    fn execute_labels_remote_action_step_as_uses() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "action.yml",
            "name: Action\non: push\njobs:\n  job:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: docker://node:20\n",
        );
        let repo = make_repo(tmp.path());
        let runtime = FakeRuntime::new();
        let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
        let config = ActRunConfig::new();
        let result = service.execute(RunActRequest::new(config, repo)).unwrap();
        assert!(result.success);
        assert_eq!(result.job_summaries[0].steps[0].step_type, StepType::Uses);
    }
}
