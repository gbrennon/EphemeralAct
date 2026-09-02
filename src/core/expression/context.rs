/// Evaluation context for `${{ }}` expressions.
///
/// Mirrors the GitHub Actions context hierarchy. Each field is a
/// `serde_json::Value` so the evaluator can traverse property and
/// index accesses naturally. Callers populate these from workflow
/// state before evaluation.
use serde_json::Value;

/// Holds all context data available during expression evaluation.
///
/// # Example
///
/// ```rust
/// use ephact::core::expression::context::EvalContext;
///
/// let ctx = EvalContext::new();
/// assert!(ctx.github.is_object());
/// ```
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// The `github` context: repository, event, actor, etc.
    pub github: Value,
    /// The `env` context: environment variables.
    pub env: Value,
    /// The `job` context: current job metadata.
    pub job: Value,
    /// The `steps` context: outputs from previous steps.
    pub steps: Value,
    /// The `runner` context: runner OS, arch, etc.
    pub runner: Value,
    /// The `secrets` context: secret values (masked in logs).
    pub secrets: Value,
    /// The `vars` context: configuration variables.
    pub vars: Value,
    /// The `strategy` context: matrix strategy info.
    pub strategy: Value,
    /// The `matrix` context: current matrix combination.
    pub matrix: Value,
    /// The `needs` context: outputs from dependent jobs.
    pub needs: Value,
    /// The `inputs` context: workflow/action inputs.
    pub inputs: Value,
}

impl Default for EvalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalContext {
    /// Creates a new evaluation context with empty objects for every field.
    #[must_use]
    pub fn new() -> Self {
        let empty_obj = Value::Object(serde_json::Map::new());
        Self {
            github: empty_obj.clone(),
            env: empty_obj.clone(),
            job: empty_obj.clone(),
            steps: empty_obj.clone(),
            runner: empty_obj.clone(),
            secrets: empty_obj.clone(),
            vars: empty_obj.clone(),
            strategy: empty_obj.clone(),
            matrix: empty_obj.clone(),
            needs: empty_obj.clone(),
            inputs: empty_obj,
        }
    }

    /// Looks up a top-level context variable by name.
    ///
    /// Returns `None` if the name does not match any known context.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        match name {
            "github" => Some(&self.github),
            "env" => Some(&self.env),
            "job" => Some(&self.job),
            "steps" => Some(&self.steps),
            "runner" => Some(&self.runner),
            "secrets" => Some(&self.secrets),
            "vars" => Some(&self.vars),
            "strategy" => Some(&self.strategy),
            "matrix" => Some(&self.matrix),
            "needs" => Some(&self.needs),
            "inputs" => Some(&self.inputs),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_objects() {
        let ctx = EvalContext::new();
        assert!(ctx.github.is_object());
        assert!(ctx.env.is_object());
    }

    #[test]
    fn get_known_context() {
        let ctx = EvalContext::new();
        assert!(ctx.get("github").is_some());
        assert!(ctx.get("env").is_some());
    }

    #[test]
    fn get_unknown_context() {
        let ctx = EvalContext::new();
        assert!(ctx.get("nonexistent").is_none());
    }

    #[test]
    fn default_equals_new() {
        let ctx1 = EvalContext::new();
        let ctx2 = EvalContext::default();
        assert_eq!(ctx1.github, ctx2.github);
    }
}
