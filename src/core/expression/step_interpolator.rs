use std::collections::HashMap;

use super::{EvalContext, EvalError, ExpressionResolver};
use crate::core::workflow::Step;

/// Produces a copy of a step with every `${{ }}` expression in its
/// user-supplied fields replaced by its evaluated value.
///
/// Interpolation covers the fields a step hands to the runner: `name`, `run`,
/// `uses`, `working-directory`, and the `with:`/`env:` maps. Control fields
/// (`if`, `continue-on-error`, `timeout-minutes`) are left as authored because
/// they are evaluated as expressions in their own right.
pub struct StepInterpolator;

impl StepInterpolator {
    /// Interpolates `step` against `context`.
    ///
    /// # Errors
    ///
    /// Returns [`EvalError`] when one of the step's expressions cannot be
    /// parsed.
    pub fn interpolate(step: &Step, context: &EvalContext) -> Result<Step, EvalError> {
        Ok(Step {
            id: step.id.clone(),
            name: Self::interpolate_field(step.name.as_deref(), context)?,
            r#if: step.r#if.clone(),
            run: Self::interpolate_field(step.run.as_deref(), context)?,
            shell: step.shell.clone(),
            working_directory: Self::interpolate_field(step.working_directory.as_deref(), context)?,
            uses: Self::interpolate_field(step.uses.as_deref(), context)?,
            with: Self::interpolate_map(&step.with, context)?,
            env: Self::interpolate_map(&step.env, context)?,
            continue_on_error: step.continue_on_error.clone(),
            timeout_minutes: step.timeout_minutes,
        })
    }

    fn interpolate_field(
        field: Option<&str>,
        context: &EvalContext,
    ) -> Result<Option<String>, EvalError> {
        field
            .map(|value| ExpressionResolver::resolve_text(value, context))
            .transpose()
    }

    fn interpolate_map(
        map: &HashMap<String, String>,
        context: &EvalContext,
    ) -> Result<HashMap<String, String>, EvalError> {
        map.iter()
            .map(|(key, value)| {
                ExpressionResolver::resolve_text(value, context)
                    .map(|resolved| (key.clone(), resolved))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn context_with_secret(name: &str, value: &str) -> EvalContext {
        let mut context = EvalContext::new();
        let mut secrets = serde_json::Map::new();
        secrets.insert(name.into(), Value::String(value.into()));
        context.secrets = Value::Object(secrets);
        context
    }

    fn step_from(yaml: &str) -> Step {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn interpolate_resolves_secrets_in_run_script() {
        let step = step_from("run: cargo publish --token ${{ secrets.TOKEN }}\n");

        let interpolated =
            StepInterpolator::interpolate(&step, &context_with_secret("TOKEN", "abc123")).unwrap();

        assert_eq!(interpolated.run(), Some("cargo publish --token abc123"));
    }

    #[test]
    fn interpolate_resolves_env_values() {
        let step = step_from("run: publish\nenv:\n  TOKEN: ${{ secrets.TOKEN }}\n");

        let interpolated =
            StepInterpolator::interpolate(&step, &context_with_secret("TOKEN", "abc123")).unwrap();

        assert_eq!(
            interpolated.env.get("TOKEN").map(String::as_str),
            Some("abc123")
        );
    }

    #[test]
    fn interpolate_resolves_with_values() {
        let mut context = EvalContext::new();
        let mut inputs = serde_json::Map::new();
        inputs.insert("mode".into(), Value::String("staging".into()));
        context.inputs = Value::Object(inputs);
        let step = step_from("uses: ./action\nwith:\n  mode: ${{ inputs.mode }}\n");

        let interpolated = StepInterpolator::interpolate(&step, &context).unwrap();

        assert_eq!(
            interpolated.with.get("mode").map(String::as_str),
            Some("staging")
        );
    }

    #[test]
    fn interpolate_resolves_action_reference() {
        let mut context = EvalContext::new();
        let mut inputs = serde_json::Map::new();
        inputs.insert("version".into(), Value::String("v4".into()));
        context.inputs = Value::Object(inputs);
        let step = step_from("uses: actions/cache@${{ inputs.version }}\n");

        let interpolated = StepInterpolator::interpolate(&step, &context).unwrap();

        assert_eq!(interpolated.uses(), Some("actions/cache@v4"));
    }

    #[test]
    fn interpolate_keeps_condition_as_authored() {
        let step = step_from("run: echo hi\nif: ${{ success() }}\n");

        let interpolated = StepInterpolator::interpolate(&step, &EvalContext::new()).unwrap();

        assert_eq!(interpolated.r#if.as_deref(), Some("${{ success() }}"));
    }

    #[test]
    fn interpolate_errors_on_unparsable_expression() {
        let step = step_from("run: echo ${{ secrets. }}\n");

        assert!(StepInterpolator::interpolate(&step, &EvalContext::new()).is_err());
    }
}
