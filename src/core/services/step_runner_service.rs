use std::{collections::HashMap, fmt, fs, path::Path};

use crate::core::{
    dtos::StepType,
    ports::outbound::{ContainerPort, ExecResult},
    workflow::{ActionDefinition, ActionRuns, Step},
};

/// Error raised while executing a workflow step, carrying any partial output
/// produced before the failure so it can be surfaced in the run summary.
#[derive(Debug)]
pub struct StepError {
    pub message: String,
    pub stdout: String,
    pub stderr: String,
}

impl StepError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    /// Returns `true` if the error message contains `needle`.
    #[cfg(test)]
    pub fn contains(&self, needle: &str) -> bool {
        self.message.contains(needle)
    }
}

impl fmt::Display for StepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Executes individual workflow steps inside a running container.
///
/// Handles four step types:
/// - `run`: executes the shell command via `bash -c`
/// - `uses: actions/checkout@*`: no-op (repo is bind-mounted into the container)
/// - `uses: ./<path>`: resolves and executes a local composite action
/// - `uses` (other actions): skipped with a warning message in stdout
pub struct StepRunnerService;

impl StepRunnerService {
    /// Executes a single step inside the given container.
    ///
    /// `repo_path` is the root of the repository, used to resolve local
    /// action references (`./` paths) relative to the workspace.
    ///
    /// `env` provides additional environment variables for this step
    /// (e.g. `GITHUB_PATH`, `GITHUB_ENV`, accumulated PATH).
    ///
    /// Returns the exec result on success, or a [`StepError`] carrying any
    /// partial output produced before the failure.
    pub fn execute(
        step: &Step,
        container: &dyn ContainerPort,
        repo_path: &Path,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, StepError> {
        match step.step_type() {
            StepType::Run => {
                let cmd = step
                    .run()
                    .ok_or_else(|| StepError::new("step has neither `run` nor `uses` defined"))?;
                Self::run_shell_command(cmd, step, container, env)
            }
            StepType::Composite | StepType::Uses => {
                let action = step
                    .uses()
                    .ok_or_else(|| StepError::new("step has neither `run` nor `uses` defined"))?;
                Self::run_action(action, step, container, repo_path, env)
            }
            StepType::Invalid => Err(StepError::new("step has neither `run` nor `uses` defined")),
        }
    }

    fn run_shell_command(
        cmd: &str,
        step: &Step,
        container: &dyn ContainerPort,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, StepError> {
        let shell = step.shell.as_deref().unwrap_or("bash");
        let cmd_parts: Vec<String> = vec![shell.to_string(), "-c".to_string(), cmd.to_string()];
        container
            .exec(&cmd_parts, step.working_directory.as_deref(), env)
            .map_err(|e| StepError::new(e.to_string()))
    }

    fn run_action(
        action: &str,
        step: &Step,
        container: &dyn ContainerPort,
        repo_path: &Path,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, StepError> {
        if action.starts_with("actions/checkout@") {
            Ok(ExecResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        } else if action.starts_with("./") {
            Self::run_local_action(action, step, container, repo_path, env)
        } else {
            Ok(ExecResult {
                exit_code: 0,
                stdout: format!("[skipped] remote action not supported: {}\n", action),
                stderr: String::new(),
            })
        }
    }

    /// Resolves and executes a local composite action (`./path/to/action`).
    ///
    /// Reads `action.yml` (or `action.yaml`) from the resolved directory,
    /// parses the action definition, and executes its steps sequentially.
    /// Only `using: composite` is supported; other run types are skipped.
    fn run_local_action(
        action: &str,
        step: &Step,
        container: &dyn ContainerPort,
        repo_path: &Path,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, StepError> {
        let action_dir = repo_path.join(action.trim_start_matches("./"));
        let action_def = Self::load_action_definition(&action_dir)
            .map_err(|e| StepError::new(format!("failed to load action '{}': {}", action, e)))?;

        match &action_def.runs {
            ActionRuns::Composite { steps } => {
                let mut stdout = String::new();
                let mut stderr = String::new();

                for action_step in steps {
                    let resolved_step = Self::resolve_inputs(action_step, &step.with);
                    match Self::execute(&resolved_step, container, repo_path, env) {
                        Ok(result) => {
                            stdout.push_str(&result.stdout);
                            stderr.push_str(&result.stderr);
                            if result.exit_code != 0 {
                                return Ok(ExecResult {
                                    exit_code: result.exit_code,
                                    stdout,
                                    stderr,
                                });
                            }
                        }
                        Err(e) => {
                            return Err(StepError {
                                message: e.message,
                                stdout: format!("{}{}", stdout, e.stdout),
                                stderr: format!("{}{}", stderr, e.stderr),
                            });
                        }
                    }
                }

                Ok(ExecResult {
                    exit_code: 0,
                    stdout,
                    stderr,
                })
            }
            _ => Ok(ExecResult {
                exit_code: 0,
                stdout: format!("[skipped] action '{}' uses unsupported run type\n", action),
                stderr: String::new(),
            }),
        }
    }

    /// Reads and parses an `action.yml` or `action.yaml` from a directory.
    fn load_action_definition(action_dir: &Path) -> Result<ActionDefinition, String> {
        let yml = action_dir.join("action.yml");
        let yaml = action_dir.join("action.yaml");

        let path = if yml.exists() {
            yml
        } else if yaml.exists() {
            yaml
        } else {
            return Err(format!("action.yml not found in {}", action_dir.display()));
        };

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        serde_yaml::from_str(&contents)
            .map_err(|e| format!("failed to parse {}: {}", path.display(), e))
    }

    /// Substitutes `${{ inputs.<name> }}` placeholders in step fields with
    /// values from the caller's `with:` map.
    ///
    /// Returns a new `Step` with resolved values. Fields without placeholders
    /// are passed through unchanged.
    fn resolve_inputs(step: &Step, with: &std::collections::HashMap<String, String>) -> Step {
        let resolve = |s: &str| -> String {
            let mut result = s.to_string();
            for (key, value) in with {
                let placeholder = format!("${{{{ inputs.{} }}}}", key);
                result = result.replace(&placeholder, value);
            }
            result
        };

        Step {
            id: step.id.clone(),
            name: step.name.clone(),
            r#if: step.r#if.clone(),
            run: step.run.as_ref().map(|r| resolve(r)),
            shell: step.shell.clone(),
            working_directory: step.working_directory.clone(),
            uses: step.uses.as_ref().map(|u| resolve(u)),
            with: step.with.clone(),
            env: step.env.clone(),
            continue_on_error: step.continue_on_error.clone(),
            timeout_minutes: step.timeout_minutes,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::core::ports::outbound::ContainerError;

    struct FakeContainer {
        last_cmd: RefCell<Vec<String>>,
        results: RefCell<Vec<ExecResult>>,
    }

    impl FakeContainer {
        fn new(results: Vec<ExecResult>) -> Self {
            Self {
                last_cmd: RefCell::new(vec![]),
                results: RefCell::new(results),
            }
        }
    }

    impl ContainerPort for FakeContainer {
        fn exec(
            &self,
            cmd: &[String],
            _workdir: Option<&str>,
            _env: &HashMap<String, String>,
        ) -> Result<ExecResult, ContainerError> {
            self.last_cmd
                .borrow_mut()
                .extend(cmd.iter().map(|s| s.to_string()));
            let mut results = self.results.borrow_mut();
            if results.is_empty() {
                Ok(ExecResult {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                })
            } else {
                Ok(results.remove(0))
            }
        }

        fn copy_to(
            &self,
            _container_path: &str,
            _entries: &[crate::core::ports::outbound::FileEntry],
        ) -> Result<(), ContainerError> {
            Ok(())
        }

        fn copy_from(
            &self,
            _container_path: &str,
        ) -> Result<Vec<crate::core::ports::outbound::FileEntry>, ContainerError> {
            Ok(vec![])
        }

        fn remove(&self) -> Result<(), ContainerError> {
            Ok(())
        }

        fn get_runner_context(
            &self,
        ) -> Result<crate::core::ports::outbound::RunnerContext, ContainerError> {
            unimplemented!()
        }
    }

    #[test]
    fn runs_shell_command_with_bash() {
        let container = FakeContainer::new(vec![ExecResult {
            exit_code: 0,
            stdout: "hello".into(),
            stderr: String::new(),
        }]);

        let yaml = "run: echo hello\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        let result =
            StepRunnerService::execute(&step, &container, Path::new("."), &HashMap::new()).unwrap();
        assert_eq!(result.stdout, "hello");

        let cmd = container.last_cmd.borrow();
        assert_eq!(cmd[0], "bash");
        assert_eq!(cmd[1], "-c");
        assert_eq!(cmd[2], "echo hello");
    }

    #[test]
    fn uses_custom_shell_when_specified() {
        let container = FakeContainer::new(vec![ExecResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);

        let yaml = "run: echo hello\nshell: python\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        StepRunnerService::execute(&step, &container, Path::new("."), &HashMap::new()).unwrap();
        let cmd = container.last_cmd.borrow();
        assert_eq!(cmd[0], "python");
    }

    #[test]
    fn checkout_action_is_noop() {
        let container = FakeContainer::new(vec![ExecResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);

        let yaml = "uses: actions/checkout@v4\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        let result =
            StepRunnerService::execute(&step, &container, Path::new("."), &HashMap::new()).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.is_empty());
    }

    #[test]
    fn unknown_action_is_skipped_with_message() {
        let container = FakeContainer::new(vec![ExecResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);

        let yaml = "uses: docker://node:20\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        let result =
            StepRunnerService::execute(&step, &container, Path::new("."), &HashMap::new()).unwrap();
        assert!(result.stdout.contains("skipped"));
        assert!(result.stdout.contains("docker://node:20"));
    }

    #[test]
    fn errors_when_step_has_neither_run_nor_uses() {
        let container = FakeContainer::new(vec![ExecResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);

        let yaml = "name: just a name\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        let err = StepRunnerService::execute(&step, &container, Path::new("."), &HashMap::new())
            .unwrap_err();
        assert!(err.contains("neither"));
    }

    #[test]
    fn executes_local_composite_action() {
        let tmp = tempfile::tempdir().unwrap();
        let action_dir = tmp
            .path()
            .join(".forgejo")
            .join("actions")
            .join("my-action");
        fs::create_dir_all(&action_dir).unwrap();
        fs::write(
            action_dir.join("action.yml"),
            r#"
name: My Action
runs:
  using: composite
  steps:
    - run: echo step1
      shell: bash
    - run: echo step2
"#,
        )
        .unwrap();

        let container = FakeContainer::new(vec![
            ExecResult {
                exit_code: 0,
                stdout: "ok".into(),
                stderr: String::new(),
            },
            ExecResult {
                exit_code: 0,
                stdout: "ok".into(),
                stderr: String::new(),
            },
        ]);

        let yaml = "uses: ./.forgejo/actions/my-action\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        let result =
            StepRunnerService::execute(&step, &container, tmp.path(), &HashMap::new()).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("ok"));

        let cmd = container.last_cmd.borrow();
        assert_eq!(cmd.len(), 6);
        assert_eq!(cmd[2], "echo step1");
        assert_eq!(cmd[5], "echo step2");
    }

    #[test]
    fn local_action_missing_yml_returns_error() {
        let tmp = tempfile::tempdir().unwrap();

        let container = FakeContainer::new(vec![ExecResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);

        let yaml = "uses: ./.forgejo/actions/nonexistent\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        let err =
            StepRunnerService::execute(&step, &container, tmp.path(), &HashMap::new()).unwrap_err();
        assert!(err.contains("action.yml not found"));
    }

    #[test]
    fn local_action_propagates_step_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let action_dir = tmp.path().join(".forgejo").join("actions").join("failing");
        fs::create_dir_all(&action_dir).unwrap();
        fs::write(
            action_dir.join("action.yml"),
            r#"
name: Failing Action
runs:
  using: composite
  steps:
    - run: echo before
    - run: exit 1
    - run: echo after
"#,
        )
        .unwrap();

        let container = FakeContainer::new(vec![
            ExecResult {
                exit_code: 0,
                stdout: "before".into(),
                stderr: String::new(),
            },
            ExecResult {
                exit_code: 1,
                stdout: "fail".into(),
                stderr: "error".into(),
            },
        ]);

        let yaml = "uses: ./.forgejo/actions/failing\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        let result =
            StepRunnerService::execute(&step, &container, tmp.path(), &HashMap::new()).unwrap();
        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.contains("fail"));
        let cmd = container.last_cmd.borrow();
        assert_eq!(cmd.len(), 6);
    }

    #[test]
    fn local_action_error_preserves_partial_output() {
        let tmp = tempfile::tempdir().unwrap();
        let action_dir = tmp.path().join(".forgejo").join("actions").join("broken");
        fs::create_dir_all(&action_dir).unwrap();
        fs::write(
            action_dir.join("action.yml"),
            r#"
name: Broken Action
runs:
  using: composite
  steps:
    - run: echo partial-output
    - name: not a runnable step
"#,
        )
        .unwrap();

        let container = FakeContainer::new(vec![ExecResult {
            exit_code: 0,
            stdout: "partial-output\n".into(),
            stderr: String::new(),
        }]);

        let yaml = "uses: ./.forgejo/actions/broken\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();

        let err =
            StepRunnerService::execute(&step, &container, tmp.path(), &HashMap::new()).unwrap_err();
        assert!(err.contains("neither"));
        assert_eq!(err.stdout, "partial-output\n");
    }

    #[test]
    fn resolve_inputs_substitutes_placeholders() {
        let yaml = r#"
run: echo "${{ inputs.path }}"
shell: bash
"#;
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        let mut with = std::collections::HashMap::new();
        with.insert("path".to_string(), "/tmp/cache".to_string());

        let resolved = StepRunnerService::resolve_inputs(&step, &with);
        assert_eq!(resolved.run(), Some("echo \"/tmp/cache\""));
    }

    #[test]
    fn resolve_inputs_handles_multiple_placeholders() {
        let yaml = r#"
run: echo "${{ inputs.key }}-${{ inputs.path }}"
"#;
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        let mut with = std::collections::HashMap::new();
        with.insert("key".to_string(), "linux".to_string());
        with.insert("path".to_string(), "/tmp".to_string());

        let resolved = StepRunnerService::resolve_inputs(&step, &with);
        assert_eq!(resolved.run(), Some("echo \"linux-/tmp\""));
    }

    #[test]
    fn resolve_inputs_noop_when_no_placeholders() {
        let yaml = "run: echo hello\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        let with = std::collections::HashMap::new();

        let resolved = StepRunnerService::resolve_inputs(&step, &with);
        assert_eq!(resolved.run(), Some("echo hello"));
    }
}
