use std::collections::HashMap;

use serde::Deserialize;

use crate::core::dtos::StepType;

/// A step in a GitHub Actions job.
///
/// Steps can be shell commands (`run`) or actions (`uses`).
/// They execute sequentially within a job and can be gated with `if`.
///
/// # Examples
///
/// ```
/// use ephact::core::workflow::Step;
///
/// // A run step
/// let yaml = "run: echo hello\nshell: bash\n";
/// let step: Step = serde_yaml::from_str(yaml).unwrap();
/// assert_eq!(step.run(), Some("echo hello"));
///
/// // A uses step
/// let yaml = "uses: actions/checkout@v4\n";
/// let step: Step = serde_yaml::from_str(yaml).unwrap();
/// assert_eq!(step.uses(), Some("actions/checkout@v4"));
/// ```
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Step {
    /// An identifier for the step (used for output references).
    pub id: Option<String>,

    /// A display name for the step.
    pub name: Option<String>,

    /// An expression that determines whether the step runs.
    #[serde(rename = "if")]
    pub r#if: Option<String>,

    /// Shell command(s) to execute.
    #[serde(default)]
    pub run: Option<String>,

    /// The shell to use for `run` commands.
    #[serde(default)]
    pub shell: Option<String>,

    /// The working directory for `run` commands.
    #[serde(rename = "working-directory")]
    pub working_directory: Option<String>,

    /// An action reference (`./`, `docker://`, or `owner/repo@ref`).
    #[serde(default)]
    pub uses: Option<String>,

    /// Input parameters for a `uses` action.
    #[serde(default)]
    pub with: HashMap<String, String>,

    /// Environment variables scoped to this step.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether to continue the job even if this step fails.
    #[serde(rename = "continue-on-error")]
    pub continue_on_error: Option<String>,

    /// Maximum number of minutes to let the step run.
    #[serde(rename = "timeout-minutes")]
    pub timeout_minutes: Option<f64>,
}

impl Step {
    /// Returns the `run` command if this is a run step.
    pub fn run(&self) -> Option<&str> {
        self.run.as_deref()
    }

    /// Returns the `uses` action reference if this is a uses step.
    pub fn uses(&self) -> Option<&str> {
        self.uses.as_deref()
    }

    /// Returns `true` if this is a run step (has `run`, no `uses`).
    pub fn is_run_step(&self) -> bool {
        self.run.is_some() && self.uses.is_none()
    }

    /// Returns `true` if this is a uses step (has `uses`, no `run`).
    pub fn is_uses_step(&self) -> bool {
        self.uses.is_some() && self.run.is_none()
    }

    /// Returns the effective shell for this step.
    ///
    /// Falls back to the default shell if none is specified.
    pub fn effective_shell<'a>(&'a self, default_shell: &'a str) -> &'a str {
        self.shell.as_deref().unwrap_or(default_shell)
    }

    /// Classifies this step: `Run` for shell commands, `Composite` for local
    /// (`./`) actions, `Uses` for other action references, and `Invalid` when
    /// the step defines neither `run` nor `uses`.
    pub fn step_type(&self) -> StepType {
        if self.run.is_some() {
            StepType::Run
        } else if self.uses.as_deref().is_some_and(|u| u.starts_with("./")) {
            StepType::Composite
        } else if self.uses.is_some() {
            StepType::Uses
        } else {
            StepType::Invalid
        }
    }

    /// Returns `true` when `continue-on-error` is set to a truthy value.
    ///
    /// The workflow parser preserves the raw scalar, so values like `True` are
    /// matched case-insensitively.
    pub fn continues_on_error(&self) -> bool {
        self.continue_on_error
            .as_deref()
            .is_some_and(|v| v.eq_ignore_ascii_case("true"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_run_step() {
        let yaml = "run: cargo test\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert!(step.is_run_step());
        assert!(!step.is_uses_step());
        assert_eq!(step.run(), Some("cargo test"));
    }

    #[test]
    fn parse_uses_step() {
        let yaml = "uses: actions/checkout@v4\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert!(step.is_uses_step());
        assert!(!step.is_run_step());
        assert_eq!(step.uses(), Some("actions/checkout@v4"));
    }

    #[test]
    fn parse_step_with_all_fields() {
        let yaml = r#"
id: test-step
name: Run tests
if: success()
run: cargo test
shell: bash
working-directory: ./src
env:
  RUST_LOG: debug
continue-on-error: true
timeout-minutes: 10
"#;
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.id.as_deref(), Some("test-step"));
        assert_eq!(step.name.as_deref(), Some("Run tests"));
        assert_eq!(step.r#if.as_deref(), Some("success()"));
        assert_eq!(step.run(), Some("cargo test"));
        assert_eq!(step.shell.as_deref(), Some("bash"));
        assert_eq!(step.working_directory.as_deref(), Some("./src"));
        assert_eq!(step.env.get("RUST_LOG").map(|s| s.as_str()), Some("debug"));
        assert_eq!(step.continue_on_error.as_deref(), Some("true"));
        assert_eq!(step.timeout_minutes, Some(10.0));
    }

    #[test]
    fn effective_shell_uses_default_when_not_set() {
        let yaml = "run: echo hello\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.effective_shell("bash"), "bash");
    }

    #[test]
    fn effective_shell_uses_step_shell_when_set() {
        let yaml = "run: echo hello\nshell: pwsh\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.effective_shell("bash"), "pwsh");
    }
}
