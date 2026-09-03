#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path, sync::Arc};

    use ephact::{
        application::{
            dtos::ExecuteActionRequest,
            ports::{
                inbound::ExecuteActionPort,
                outbound::{
                    ContainerConfig, ContainerPort, ContainerRuntimePort, ExecResult, RunnerContext,
                },
            },
        },
        domain::{expression::EvalContext, workflow::Step},
        infrastructure::ActionExecutionWiring,
    };
    use serde_json::Value;

    use crate::common::fakes::{
        fake_action_fetcher::FakeActionFetcher, fake_runtime::FakeRuntime,
        stub_failing_action_fetcher::StubFailingActionFetcher,
    };

    fn container(runtime: &FakeRuntime) -> Arc<dyn ContainerPort> {
        Arc::from(
            runtime
                .create_container(&ContainerConfig {
                    image: "image".into(),
                    platform: None,
                    env: HashMap::new(),
                    binds: vec![],
                    workdir: None,
                    cmd: None,
                    entrypoint: None,
                    network: None,
                    name: None,
                    runner_context: RunnerContext::default(),
                })
                .unwrap(),
        )
    }

    fn step_from(yaml: &str) -> Step {
        serde_yaml::from_str(yaml).unwrap()
    }

    fn request(
        action_ref: &str,
        step: Step,
        repo_path: &Path,
        container: Arc<dyn ContainerPort>,
        context: EvalContext,
    ) -> ExecuteActionRequest {
        ExecuteActionRequest {
            action_ref: action_ref.to_string(),
            step,
            repo_path: repo_path.to_path_buf(),
            env: HashMap::new(),
            context,
            container,
        }
    }

    fn write_action(dir: &Path, body: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("action.yml"), body).unwrap();
    }

    fn push_result(runtime: &FakeRuntime, exit_code: i64, stdout: &str) {
        runtime.exec_results.borrow_mut().push(ExecResult {
            exit_code,
            stdout: stdout.into(),
            stderr: String::new(),
        });
    }

    #[test]
    fn execute_runs_the_steps_of_a_local_composite_action() {
        let repo = tempfile::tempdir().unwrap();
        write_action(
            &repo.path().join("actions/greet"),
            "name: Greet\nruns:\n  using: composite\n  steps:\n    - run: echo hi\n      shell: bash\n",
        );
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "hi\n");
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));

        let response = service
            .execute(request(
                "./actions/greet",
                step_from("uses: ./actions/greet\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout, "hi\n");
        assert_eq!(runtime.executed_scripts(), vec!["echo hi".to_string()]);
    }

    #[test]
    fn execute_passes_step_inputs_to_composite_expressions() {
        let repo = tempfile::tempdir().unwrap();
        write_action(
            &repo.path().join("actions/deploy"),
            "name: Deploy\ninputs:\n  mode:\n    description: target\n    default: production\nruns:\n  using: composite\n  steps:\n    - run: deploy ${{ inputs.mode }}\n      shell: bash\n",
        );
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));

        service
            .execute(request(
                "./actions/deploy",
                step_from("uses: ./actions/deploy\nwith:\n  mode: staging\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(
            runtime.executed_scripts(),
            vec!["deploy staging".to_string()]
        );
    }

    #[test]
    fn execute_falls_back_to_declared_input_defaults() {
        let repo = tempfile::tempdir().unwrap();
        write_action(
            &repo.path().join("actions/deploy"),
            "name: Deploy\ninputs:\n  mode:\n    description: target\n    default: production\nruns:\n  using: composite\n  steps:\n    - run: deploy ${{ inputs.mode }}\n      shell: bash\n",
        );
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));

        service
            .execute(request(
                "./actions/deploy",
                step_from("uses: ./actions/deploy\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(
            runtime.executed_scripts(),
            vec!["deploy production".to_string()]
        );
    }

    #[test]
    fn execute_resolves_secrets_from_the_run_context() {
        let repo = tempfile::tempdir().unwrap();
        write_action(
            &repo.path().join("actions/publish"),
            "name: Publish\nruns:\n  using: composite\n  steps:\n    - run: publish --token ${{ secrets.TOKEN }}\n      shell: bash\n",
        );
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));
        let mut context = EvalContext::new();
        let mut secrets = serde_json::Map::new();
        secrets.insert("TOKEN".into(), Value::String("abc123".into()));
        context.secrets = Value::Object(secrets);

        service
            .execute(request(
                "./actions/publish",
                step_from("uses: ./actions/publish\n"),
                repo.path(),
                container(&runtime),
                context,
            ))
            .unwrap();

        assert_eq!(
            runtime.executed_scripts(),
            vec!["publish --token abc123".to_string()]
        );
    }

    #[test]
    fn execute_stops_composite_action_at_the_first_failing_step() {
        let repo = tempfile::tempdir().unwrap();
        write_action(
            &repo.path().join("actions/build"),
            "name: Build\nruns:\n  using: composite\n  steps:\n    - run: first\n      shell: bash\n    - run: second\n      shell: bash\n",
        );
        let runtime = FakeRuntime::new();
        push_result(&runtime, 2, "boom\n");
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));

        let response = service
            .execute(request(
                "./actions/build",
                step_from("uses: ./actions/build\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(response.exit_code, 2);
        assert_eq!(runtime.executed_scripts(), vec!["first".to_string()]);
    }

    #[test]
    fn execute_fetches_a_remote_action_and_runs_it() {
        let repo = tempfile::tempdir().unwrap();
        let mirror = tempfile::tempdir().unwrap();
        write_action(
            mirror.path(),
            "name: Cache\nruns:\n  using: composite\n  steps:\n    - run: restore-cache\n      shell: bash\n",
        );
        let runtime = FakeRuntime::new();
        let fetcher = FakeActionFetcher::returning(mirror.path().into());
        let service = ActionExecutionWiring::build(Box::new(fetcher));

        let response = service
            .execute(request(
                "https://data.forgejo.org/actions/cache@v4",
                step_from("uses: https://data.forgejo.org/actions/cache@v4\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(response.exit_code, 0);
        assert_eq!(
            runtime.executed_scripts(),
            vec!["restore-cache".to_string()]
        );
    }

    #[test]
    fn execute_runs_a_javascript_action_with_inputs_as_environment_variables() {
        let repo = tempfile::tempdir().unwrap();
        let mirror = tempfile::tempdir().unwrap();
        write_action(
            mirror.path(),
            "name: Cache\nruns:\n  using: node20\n  main: dist/index.js\n",
        );
        std::fs::create_dir_all(mirror.path().join("dist")).unwrap();
        std::fs::write(mirror.path().join("dist/index.js"), "console.log('cached')").unwrap();
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            mirror.path().into(),
        )));

        let response = service
            .execute(request(
                "https://data.forgejo.org/actions/cache@v4",
                step_from(
                    "uses: https://data.forgejo.org/actions/cache@v4\nwith:\n  key: build-cache\n",
                ),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(response.exit_code, 0);
        let commands = runtime.executed_commands.borrow();
        let node_command = commands
            .iter()
            .find(|command| command.first().map(String::as_str) == Some("node"))
            .expect("the action entry point should run with node");
        assert!(
            node_command[1].ends_with("/dist/index.js"),
            "{:?}",
            node_command
        );
        assert!(
            runtime
                .exec_environments
                .borrow()
                .iter()
                .any(|env| { env.get("INPUT_KEY").map(String::as_str) == Some("build-cache") }),
            "inputs should reach the action as INPUT_* variables"
        );
        assert_eq!(runtime.copied_paths.borrow().len(), 1);
    }

    #[test]
    fn execute_runs_a_javascript_action_with_the_interpreter_found_in_the_container() {
        let repo = tempfile::tempdir().unwrap();
        let mirror = tempfile::tempdir().unwrap();
        write_action(
            mirror.path(),
            "name: Cache\nruns:\n  using: node20\n  main: dist/index.js\n",
        );
        std::fs::create_dir_all(mirror.path().join("dist")).unwrap();
        std::fs::write(mirror.path().join("dist/index.js"), "console.log('cached')").unwrap();
        let runtime = FakeRuntime::new();
        push_result(&runtime, 0, "");
        push_result(&runtime, 0, "/opt/acttoolcache/node/24/bin/node\n");
        push_result(&runtime, 0, "cached\n");
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            mirror.path().into(),
        )));

        let response = service
            .execute(request(
                "https://data.forgejo.org/actions/cache@v4",
                step_from("uses: https://data.forgejo.org/actions/cache@v4\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(response.stdout, "cached\n");
        assert!(
            runtime
                .executed_commands
                .borrow()
                .iter()
                .any(|command| command.first().map(String::as_str)
                    == Some("/opt/acttoolcache/node/24/bin/node")),
            "{:?}",
            runtime.executed_commands.borrow()
        );
    }

    #[test]
    fn execute_skips_checkout_because_the_workspace_is_mounted() {
        let repo = tempfile::tempdir().unwrap();
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));

        let response = service
            .execute(request(
                "actions/checkout@v4",
                step_from("uses: actions/checkout@v4\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(response.exit_code, 0);
        assert!(response.stdout.contains("[skipped]"), "{}", response.stdout);
        assert!(runtime.executed_commands.borrow().is_empty());
    }

    #[test]
    fn execute_reports_a_failed_fetch() {
        let repo = tempfile::tempdir().unwrap();
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(StubFailingActionFetcher));

        let error = service
            .execute(request(
                "https://data.forgejo.org/actions/cache@v4",
                step_from("uses: https://data.forgejo.org/actions/cache@v4\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap_err();

        assert!(
            error.message.contains("failed to fetch action"),
            "{}",
            error.message
        );
    }

    #[test]
    fn execute_reports_container_actions_as_unsupported() {
        let repo = tempfile::tempdir().unwrap();
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));

        let error = service
            .execute(request(
                "docker://node:20",
                step_from("uses: docker://node:20\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap_err();

        assert!(
            error.message.contains("unsupported action"),
            "{}",
            error.message
        );
    }

    #[test]
    fn execute_reports_a_missing_action_definition() {
        let repo = tempfile::tempdir().unwrap();
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));

        let error = service
            .execute(request(
                "./actions/absent",
                step_from("uses: ./actions/absent\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap_err();

        assert!(
            error.message.contains("action.yml not found"),
            "{}",
            error.message
        );
    }

    #[test]
    fn execute_runs_actions_nested_inside_a_composite_action() {
        let repo = tempfile::tempdir().unwrap();
        write_action(
            &repo.path().join("actions/outer"),
            "name: Outer\nruns:\n  using: composite\n  steps:\n    - uses: ./actions/inner\n",
        );
        write_action(
            &repo.path().join("actions/inner"),
            "name: Inner\nruns:\n  using: composite\n  steps:\n    - run: inner-step\n      shell: bash\n",
        );
        let runtime = FakeRuntime::new();
        let service = ActionExecutionWiring::build(Box::new(FakeActionFetcher::returning(
            repo.path().into(),
        )));

        let response = service
            .execute(request(
                "./actions/outer",
                step_from("uses: ./actions/outer\n"),
                repo.path(),
                container(&runtime),
                EvalContext::new(),
            ))
            .unwrap();

        assert_eq!(response.exit_code, 0);
        assert_eq!(runtime.executed_scripts(), vec!["inner-step".to_string()]);
    }
}
