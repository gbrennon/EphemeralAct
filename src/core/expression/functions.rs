/// Built-in functions for GitHub Actions `${{ }}` expressions.
///
/// Implements the standard function library: `contains`, `startsWith`,
/// `endsWith`, `format`, `join`, `toJson`, `fromJson`, and the
/// status-check functions `success`, `always`, `cancelled`, `failure`.
use serde_json::Value;
use thiserror::Error;

use super::context::EvalContext;

/// Errors that can occur during expression function evaluation.
#[derive(Error, Debug, PartialEq)]
pub enum EvalError {
    /// A function received an argument of the wrong type.
    #[error("type error: {0}")]
    TypeError(String),

    /// A function received the wrong number of arguments.
    #[error("argument count error: {0}")]
    ArgCount(String),

    /// A format string could not be parsed or applied.
    #[error("format error: {0}")]
    FormatError(String),

    /// A JSON value could not be parsed or serialized.
    #[error("JSON error: {0}")]
    JsonError(String),
}

/// Built-in function dispatcher for GitHub Actions expressions.
///
/// Wraps an [`EvalContext`] reference and exposes each built-in
/// as a method, plus a generic [`call`](Self::call) dispatcher
/// that routes by function name (case-insensitive).
pub struct Functions<'a> {
    /// Reference to the current evaluation context.
    #[allow(dead_code)]
    context: &'a EvalContext,
}

impl<'a> Functions<'a> {
    /// Creates a new `Functions` dispatcher bound to the given context.
    #[must_use]
    pub fn new(context: &'a EvalContext) -> Self {
        Self { context }
    }

    // ------------------------------------------------------------------
    // String / array helpers
    // ------------------------------------------------------------------

    /// Returns `true` if `search` contains `item`.
    ///
    /// - **String search**: case-insensitive substring match.
    /// - **Array search**: exact element match (using `serde_json` equality).
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::TypeError`] if `search` is neither a string
    /// nor an array.
    pub fn contains(&self, search: &Value, item: &Value) -> Result<Value, EvalError> {
        match search {
            Value::String(s) => {
                let item_str = item.as_str().ok_or_else(|| {
                    EvalError::TypeError(
                        "contains: item must be a string when search is a string".into(),
                    )
                })?;
                Ok(Value::Bool(
                    s.to_lowercase().contains(&item_str.to_lowercase()),
                ))
            }
            Value::Array(arr) => Ok(Value::Bool(arr.contains(item))),
            other => Err(EvalError::TypeError(format!(
                "contains: search must be a string or array, got {}",
                value_type_name(other)
            ))),
        }
    }

    /// Returns `true` if the string `search` starts with `prefix`
    /// (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::TypeError`] if either argument is not a string.
    pub fn starts_with(&self, search: &Value, prefix: &Value) -> Result<Value, EvalError> {
        let s = expect_string(search, "startsWith", "search")?;
        let p = expect_string(prefix, "startsWith", "prefix")?;
        Ok(Value::Bool(s.to_lowercase().starts_with(&p.to_lowercase())))
    }

    /// Returns `true` if the string `search` ends with `suffix`
    /// (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::TypeError`] if either argument is not a string.
    pub fn ends_with(&self, search: &Value, suffix: &Value) -> Result<Value, EvalError> {
        let s = expect_string(search, "endsWith", "search")?;
        let sfx = expect_string(suffix, "endsWith", "suffix")?;
        Ok(Value::Bool(s.to_lowercase().ends_with(&sfx.to_lowercase())))
    }

    // ------------------------------------------------------------------
    // Formatting
    // ------------------------------------------------------------------

    /// Formats a template string by replacing `{0}`, `{1}`, … with the
    /// string representations of the positional arguments.
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::TypeError`] if `template` is not a string.
    /// Returns [`EvalError::FormatError`] if a placeholder index is
    /// out of range or malformed.
    pub fn format(&self, template: &Value, args: &[Value]) -> Result<Value, EvalError> {
        let tmpl = expect_string(template, "format", "template")?;
        let mut result = String::with_capacity(tmpl.len());
        let mut rest = tmpl;
        let mut chars = rest.char_indices();

        while let Some((i, ch)) = chars.next() {
            if ch == '{' {
                // Collect digits until '}'
                let start = i + 1; // after '{'
                let mut end = start;
                let mut found_close = false;
                for (j, c) in chars.by_ref() {
                    if c == '}' {
                        found_close = true;
                        end = j;
                        break;
                    }
                    if !c.is_ascii_digit() {
                        return Err(EvalError::FormatError(format!(
                            "format: invalid placeholder character '{c}' at position {j}"
                        )));
                    }
                }
                if !found_close {
                    return Err(EvalError::FormatError(
                        "format: unclosed placeholder".into(),
                    ));
                }
                let idx_str = &rest[start..end];
                let idx: usize = idx_str.parse().map_err(|_| {
                    EvalError::FormatError(format!("format: invalid placeholder index '{idx_str}'"))
                })?;
                let replacement = args.get(idx).ok_or_else(|| {
                    EvalError::FormatError(format!(
                        "format: placeholder index {idx} out of range (have {} args)",
                        args.len()
                    ))
                })?;
                result.push_str(&value_to_string(replacement));
                rest = &rest[end + 1..]; // after '}'
                chars = rest.char_indices();
            } else if ch == '}' {
                return Err(EvalError::FormatError(
                    "format: unexpected '}' without opening '{'".into(),
                ));
            } else {
                result.push(ch);
            }
        }

        Ok(Value::String(result))
    }

    /// Joins the elements of `array` into a single string, separated by
    /// `sep`.
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::TypeError`] if `array` is not an array or
    /// `sep` is not a string.
    pub fn join(&self, array: &Value, sep: &Value) -> Result<Value, EvalError> {
        let arr = array
            .as_array()
            .ok_or_else(|| EvalError::TypeError("join: first argument must be an array".into()))?;
        let separator = expect_string(sep, "join", "separator")?;
        let parts: Vec<String> = arr.iter().map(value_to_string).collect();
        Ok(Value::String(parts.join(separator)))
    }

    // ------------------------------------------------------------------
    // JSON
    // ------------------------------------------------------------------

    /// Serializes `value` to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::JsonError`] if serialization fails.
    pub fn to_json(&self, value: &Value) -> Result<Value, EvalError> {
        serde_json::to_string(value)
            .map(Value::String)
            .map_err(|e| EvalError::JsonError(format!("toJson: {e}")))
    }

    /// Parses a JSON string into a `Value`.
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::TypeError`] if `value` is not a string.
    /// Returns [`EvalError::JsonError`] if parsing fails.
    pub fn from_json(&self, value: &Value) -> Result<Value, EvalError> {
        let s = expect_string(value, "fromJson", "value")?;
        serde_json::from_str(s).map_err(|e| EvalError::JsonError(format!("fromJson: {e}")))
    }

    // ------------------------------------------------------------------
    // Status check stubs
    // ------------------------------------------------------------------

    /// Always returns `true` — stub for job status check.
    ///
    /// In a full implementation this would consult the workflow context
    /// to determine whether all previous steps succeeded.
    pub fn success(&self) -> Result<Value, EvalError> {
        Ok(Value::Bool(true))
    }

    /// Always returns `true` — stub for unconditional execution check.
    pub fn always(&self) -> Result<Value, EvalError> {
        Ok(Value::Bool(true))
    }

    /// Always returns `false` — stub for cancellation check.
    pub fn cancelled(&self) -> Result<Value, EvalError> {
        Ok(Value::Bool(false))
    }

    /// Always returns `false` — stub for failure check.
    pub fn failure(&self) -> Result<Value, EvalError> {
        Ok(Value::Bool(false))
    }

    // ------------------------------------------------------------------
    // Dispatch
    // ------------------------------------------------------------------

    /// Dispatches a function call by name (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::ArgCount`] if the wrong number of arguments
    /// is supplied. Returns [`EvalError::TypeError`] or other variants
    /// as propagated from the individual function implementations.
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, EvalError> {
        match name.to_lowercase().as_str() {
            "contains" => {
                expect_arg_count(name, args.len(), 2, 2)?;
                self.contains(&args[0], &args[1])
            }
            "startswith" => {
                expect_arg_count(name, args.len(), 2, 2)?;
                self.starts_with(&args[0], &args[1])
            }
            "endswith" => {
                expect_arg_count(name, args.len(), 2, 2)?;
                self.ends_with(&args[0], &args[1])
            }
            "format" => {
                if args.is_empty() {
                    return Err(EvalError::ArgCount(format!(
                        "{name}: expected at least 1 argument, got 0"
                    )));
                }
                self.format(&args[0], &args[1..])
            }
            "join" => {
                expect_arg_count(name, args.len(), 2, 2)?;
                self.join(&args[0], &args[1])
            }
            "tojson" => {
                expect_arg_count(name, args.len(), 1, 1)?;
                self.to_json(&args[0])
            }
            "fromjson" => {
                expect_arg_count(name, args.len(), 1, 1)?;
                self.from_json(&args[0])
            }
            "success" => {
                expect_arg_count(name, args.len(), 0, 0)?;
                self.success()
            }
            "always" => {
                expect_arg_count(name, args.len(), 0, 0)?;
                self.always()
            }
            "cancelled" => {
                expect_arg_count(name, args.len(), 0, 0)?;
                self.cancelled()
            }
            "failure" => {
                expect_arg_count(name, args.len(), 0, 0)?;
                self.failure()
            }
            other => Err(EvalError::TypeError(format!("unknown function: {other}"))),
        }
    }
}

// ------------------------------------------------------------------
// Free helper functions
// ------------------------------------------------------------------

/// Extracts a string reference from a `Value`, or returns a type error.
fn expect_string<'v>(value: &'v Value, func: &str, arg_name: &str) -> Result<&'v str, EvalError> {
    value.as_str().ok_or_else(|| {
        EvalError::TypeError(format!(
            "{func}: {arg_name} must be a string, got {}",
            value_type_name(value)
        ))
    })
}

/// Returns a human-readable name for a `Value` variant.
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Converts a `Value` to its string representation for `format` / `join`.
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

/// Validates that `actual` arg count falls within `[min, max]`.
fn expect_arg_count(func: &str, actual: usize, min: usize, max: usize) -> Result<(), EvalError> {
    if actual < min || actual > max {
        let expected = if min == max {
            min.to_string()
        } else {
            format!("{min}..{max}")
        };
        return Err(EvalError::ArgCount(format!(
            "{func}: expected {expected} argument(s), got {actual}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx() -> EvalContext {
        EvalContext::new()
    }

    fn fns(ctx: &EvalContext) -> Functions<'_> {
        Functions::new(ctx)
    }

    // -- contains -------------------------------------------------------

    #[test]
    fn contains_string_match_case_insensitive() {
        let c = ctx();
        let f = fns(&c);
        let result = f.contains(&json!("Hello World"), &json!("world")).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn contains_string_no_match() {
        let c = ctx();
        let f = fns(&c);
        let result = f.contains(&json!("Hello World"), &json!("xyz")).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn contains_array_match() {
        let c = ctx();
        let f = fns(&c);
        let result = f.contains(&json!(["a", "b", "c"]), &json!("b")).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn contains_array_no_match() {
        let c = ctx();
        let f = fns(&c);
        let result = f.contains(&json!(["a", "b"]), &json!("c")).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn contains_type_error_on_number() {
        let c = ctx();
        let f = fns(&c);
        let err = f.contains(&json!(42), &json!("x")).unwrap_err();
        assert!(matches!(err, EvalError::TypeError(_)));
    }

    // -- startsWith -----------------------------------------------------

    #[test]
    fn starts_with_match() {
        let c = ctx();
        let f = fns(&c);
        let result = f
            .starts_with(&json!("Hello World"), &json!("hello"))
            .unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn starts_with_no_match() {
        let c = ctx();
        let f = fns(&c);
        let result = f
            .starts_with(&json!("Hello World"), &json!("World"))
            .unwrap();
        assert_eq!(result, json!(false));
    }

    // -- endsWith -------------------------------------------------------

    #[test]
    fn ends_with_match() {
        let c = ctx();
        let f = fns(&c);
        let result = f.ends_with(&json!("Hello World"), &json!("WORLD")).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn ends_with_no_match() {
        let c = ctx();
        let f = fns(&c);
        let result = f.ends_with(&json!("Hello World"), &json!("Hello")).unwrap();
        assert_eq!(result, json!(false));
    }

    // -- format ---------------------------------------------------------

    #[test]
    fn format_basic() {
        let c = ctx();
        let f = fns(&c);
        let result = f.format(&json!("Hello {0}"), &[json!("world")]).unwrap();
        assert_eq!(result, json!("Hello world"));
    }

    #[test]
    fn format_multiple_args() {
        let c = ctx();
        let f = fns(&c);
        let result = f
            .format(&json!("{0} + {1} = {2}"), &[json!(1), json!(2), json!(3)])
            .unwrap();
        assert_eq!(result, json!("1 + 2 = 3"));
    }

    #[test]
    fn format_no_placeholders() {
        let c = ctx();
        let f = fns(&c);
        let result = f.format(&json!("no placeholders"), &[]).unwrap();
        assert_eq!(result, json!("no placeholders"));
    }

    #[test]
    fn format_index_out_of_range() {
        let c = ctx();
        let f = fns(&c);
        let err = f
            .format(&json!("Hello {5}"), &[json!("world")])
            .unwrap_err();
        assert!(matches!(err, EvalError::FormatError(_)));
    }

    // -- join -----------------------------------------------------------

    #[test]
    fn join_basic() {
        let c = ctx();
        let f = fns(&c);
        let result = f.join(&json!(["a", "b", "c"]), &json!(", ")).unwrap();
        assert_eq!(result, json!("a, b, c"));
    }

    #[test]
    fn join_single_element() {
        let c = ctx();
        let f = fns(&c);
        let result = f.join(&json!(["only"]), &json!(", ")).unwrap();
        assert_eq!(result, json!("only"));
    }

    #[test]
    fn join_empty_array() {
        let c = ctx();
        let f = fns(&c);
        let result = f.join(&json!([]), &json!(", ")).unwrap();
        assert_eq!(result, json!(""));
    }

    // -- toJson / fromJson ----------------------------------------------

    #[test]
    fn to_json_roundtrip() {
        let c = ctx();
        let f = fns(&c);
        let original = json!({"key": "value", "num": 42});
        let json_str = f.to_json(&original).unwrap();
        let parsed = f.from_json(&json_str).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn from_json_invalid() {
        let c = ctx();
        let f = fns(&c);
        let err = f.from_json(&json!("not json")).unwrap_err();
        assert!(matches!(err, EvalError::JsonError(_)));
    }

    // -- Status stubs ---------------------------------------------------

    #[test]
    fn success_returns_true() {
        let c = ctx();
        let f = fns(&c);
        assert_eq!(f.success().unwrap(), json!(true));
    }

    #[test]
    fn always_returns_true() {
        let c = ctx();
        let f = fns(&c);
        assert_eq!(f.always().unwrap(), json!(true));
    }

    #[test]
    fn cancelled_returns_false() {
        let c = ctx();
        let f = fns(&c);
        assert_eq!(f.cancelled().unwrap(), json!(false));
    }

    #[test]
    fn failure_returns_false() {
        let c = ctx();
        let f = fns(&c);
        assert_eq!(f.failure().unwrap(), json!(false));
    }

    // -- call dispatch --------------------------------------------------

    #[test]
    fn call_unknown_function_error() {
        let c = ctx();
        let f = fns(&c);
        let err = f.call("nonexistent", &[]).unwrap_err();
        assert!(matches!(err, EvalError::TypeError(_)));
    }

    #[test]
    fn call_wrong_arg_count() {
        let c = ctx();
        let f = fns(&c);
        let err = f.call("contains", &[json!("only one")]).unwrap_err();
        assert!(matches!(err, EvalError::ArgCount(_)));
    }

    #[test]
    fn call_case_insensitive() {
        let c = ctx();
        let f = fns(&c);
        // "CoNtAiNs" should still dispatch to contains
        let result = f
            .call("CoNtAiNs", &[json!("Hello World"), json!("world")])
            .unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn call_success_zero_args() {
        let c = ctx();
        let f = fns(&c);
        let result = f.call("success", &[]).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn call_success_with_args_is_error() {
        let c = ctx();
        let f = fns(&c);
        let err = f.call("success", &[json!("extra")]).unwrap_err();
        assert!(matches!(err, EvalError::ArgCount(_)));
    }
}
