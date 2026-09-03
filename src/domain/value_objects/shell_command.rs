use std::collections::HashMap;

use crate::domain::workflow::Step;

/// Shell used when a step declares no `shell:`.
const DEFAULT_SHELL: &str = "bash";

/// The invocation a `run:` step turns into: an argument vector, the directory
/// it runs in, and the environment it sees.
///
/// The step is expected to be interpolated already - building a command never
/// evaluates `${{ }}` expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommand {
    argv: Vec<String>,
    working_directory: Option<String>,
    env: HashMap<String, String>,
}

impl ShellCommand {
    /// Builds the invocation for a step, layering the step's own `env:` on top
    /// of the job environment.
    ///
    /// Returns `None` when the step declares no script to run.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use ephact::domain::{value_objects::ShellCommand, workflow::Step};
    /// let step: Step = serde_yaml::from_str("run: echo hi\n").unwrap();
    /// let command = ShellCommand::for_step(&step, &HashMap::new()).unwrap();
    /// assert_eq!(command.argv(), ["bash", "-c", "echo hi"]);
    /// ```
    pub fn for_step(step: &Step, job_env: &HashMap<String, String>) -> Option<Self> {
        let script = step.run()?;
        let shell = step.effective_shell(DEFAULT_SHELL);

        let mut env = job_env.clone();
        env.extend(
            step.env
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );

        Some(Self {
            argv: vec![shell.to_string(), "-c".to_string(), script.to_string()],
            working_directory: step.working_directory.clone(),
            env,
        })
    }

    /// Builds an invocation from an explicit argument vector.
    pub fn new(
        argv: Vec<String>,
        working_directory: Option<String>,
        env: HashMap<String, String>,
    ) -> Self {
        Self {
            argv,
            working_directory,
            env,
        }
    }

    /// Returns the argument vector to execute.
    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    /// Returns the directory the command runs in, if the step pinned one.
    pub fn working_directory(&self) -> Option<&str> {
        self.working_directory.as_deref()
    }

    /// Returns the environment the command runs with.
    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step_from(yaml: &str) -> Step {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn for_step_defaults_to_bash() {
        let command =
            ShellCommand::for_step(&step_from("run: echo hi\n"), &HashMap::new()).unwrap();

        assert_eq!(command.argv(), ["bash", "-c", "echo hi"]);
    }

    #[test]
    fn for_step_honors_declared_shell() {
        let command =
            ShellCommand::for_step(&step_from("run: echo hi\nshell: sh\n"), &HashMap::new())
                .unwrap();

        assert_eq!(command.argv(), ["sh", "-c", "echo hi"]);
    }

    #[test]
    fn for_step_keeps_working_directory() {
        let command = ShellCommand::for_step(
            &step_from("run: echo hi\nworking-directory: crates/app\n"),
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(command.working_directory(), Some("crates/app"));
    }

    #[test]
    fn for_step_layers_step_env_over_job_env() {
        let mut job_env = HashMap::new();
        job_env.insert("MODE".to_string(), "job".to_string());
        job_env.insert("KEEP".to_string(), "yes".to_string());

        let command =
            ShellCommand::for_step(&step_from("run: echo hi\nenv:\n  MODE: step\n"), &job_env)
                .unwrap();

        assert_eq!(command.env().get("MODE").map(String::as_str), Some("step"));
        assert_eq!(command.env().get("KEEP").map(String::as_str), Some("yes"));
    }

    #[test]
    fn for_step_returns_none_without_script() {
        assert!(ShellCommand::for_step(&step_from("uses: ./action\n"), &HashMap::new()).is_none());
    }

    #[test]
    fn new_builds_an_explicit_invocation() {
        let command = ShellCommand::new(
            vec!["node".into(), "/actions/main.js".into()],
            Some("/workspace".into()),
            HashMap::new(),
        );

        assert_eq!(command.argv(), ["node", "/actions/main.js"]);
        assert_eq!(command.working_directory(), Some("/workspace"));
        assert!(command.env().is_empty());
    }
}
