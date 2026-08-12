/// AST evaluator for GitHub Actions `${{ }}` expressions.
///
/// Walks an [`Expr`] AST and produces a [`serde_json::Value`] result,
/// using the built-in [`Functions`] dispatcher for function calls.
use serde_json::Value;

use super::{CompareOp, EvalContext, EvalError, Expr, Functions, Literal, LogicalOp};

/// Walks an expression AST and evaluates it to a [`Value`].
pub struct Evaluator<'a> {
    context: &'a EvalContext,
    functions: Functions<'a>,
}

impl<'a> Evaluator<'a> {
    /// Creates a new `Evaluator` bound to the given evaluation context.
    #[must_use]
    pub fn new(context: &'a EvalContext) -> Self {
        Self {
            context,
            functions: Functions::new(context),
        }
    }

    /// Evaluates an expression AST node, returning the resulting [`Value`].
    ///
    /// # Errors
    ///
    /// Returns [`EvalError`] for type errors, unknown functions, or
    /// other evaluation failures.
    pub fn evaluate(&self, expr: &Expr) -> Result<Value, EvalError> {
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Variable(name) => self.eval_variable(name),
            Expr::PropertyAccess(obj, prop) => self.eval_property_access(obj, prop),
            Expr::IndexAccess(obj, idx) => self.eval_index_access(obj, idx),
            Expr::ArrayDeref(obj) => self.eval_array_deref(obj),
            Expr::Not(inner) => self.eval_not(inner),
            Expr::Compare(op, left, right) => self.eval_compare(*op, left, right),
            Expr::Logical(op, left, right) => self.eval_logical(*op, left, right),
            Expr::FuncCall(name, args) => self.eval_func_call(name, args),
        }
    }

    fn eval_literal(&self, lit: &Literal) -> Result<Value, EvalError> {
        match lit {
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Null => Ok(Value::Null),
            Literal::Int(n) => Ok(Value::Number((*n).into())),
            Literal::Float(f) => serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .ok_or_else(|| EvalError::TypeError(format!("invalid float literal: {f}"))),
            Literal::String(s) => Ok(Value::String(s.clone())),
        }
    }

    fn eval_variable(&self, name: &str) -> Result<Value, EvalError> {
        if let Some(val) = self.context.get(name) {
            return Ok(val.clone());
        }
        Ok(Value::String(format!("${{{{ {name} }}}}")))
    }

    fn eval_property_access(&self, obj: &Expr, prop: &str) -> Result<Value, EvalError> {
        let val = self.evaluate(obj)?;
        match val {
            Value::Object(map) => map.get(prop).cloned().ok_or_else(|| {
                EvalError::TypeError(format!("property '{prop}' not found on object"))
            }),
            _ => Err(EvalError::TypeError(format!(
                "cannot access property '{prop}' on non-object value"
            ))),
        }
    }

    fn eval_index_access(&self, obj: &Expr, idx: &Expr) -> Result<Value, EvalError> {
        let obj_val = self.evaluate(obj)?;
        let idx_val = self.evaluate(idx)?;
        match (&obj_val, &idx_val) {
            (Value::Array(arr), Value::Number(n)) => {
                let i = n.as_u64().ok_or_else(|| {
                    EvalError::TypeError("index must be a non-negative integer".into())
                })? as usize;
                arr.get(i).cloned().ok_or_else(|| {
                    EvalError::TypeError(format!(
                        "index {i} out of bounds for array of length {}",
                        arr.len()
                    ))
                })
            }
            (Value::Object(map), Value::String(key)) => map
                .get(key.as_str())
                .cloned()
                .ok_or_else(|| EvalError::TypeError(format!("key '{key}' not found on object"))),
            _ => Err(EvalError::TypeError(
                "index access requires array+number or object+string".into(),
            )),
        }
    }

    fn eval_array_deref(&self, obj: &Expr) -> Result<Value, EvalError> {
        self.evaluate(obj)
    }

    fn eval_not(&self, inner: &Expr) -> Result<Value, EvalError> {
        let val = self.evaluate(inner)?;
        Ok(Value::Bool(!is_truthy(&val)))
    }

    fn eval_compare(&self, op: CompareOp, left: &Expr, right: &Expr) -> Result<Value, EvalError> {
        let lhs = self.evaluate(left)?;
        let rhs = self.evaluate(right)?;
        let ordering = compare_values(&lhs, &rhs)?;
        let result = match op {
            CompareOp::Eq => ordering == std::cmp::Ordering::Equal,
            CompareOp::Neq => ordering != std::cmp::Ordering::Equal,
            CompareOp::Lt => ordering == std::cmp::Ordering::Less,
            CompareOp::Lte => ordering != std::cmp::Ordering::Greater,
            CompareOp::Gt => ordering == std::cmp::Ordering::Greater,
            CompareOp::Gte => ordering != std::cmp::Ordering::Less,
        };
        Ok(Value::Bool(result))
    }

    fn eval_logical(&self, op: LogicalOp, left: &Expr, right: &Expr) -> Result<Value, EvalError> {
        let lhs = self.evaluate(left)?;
        match op {
            LogicalOp::And => {
                if !is_truthy(&lhs) {
                    return Ok(lhs);
                }
            }
            LogicalOp::Or => {
                if is_truthy(&lhs) {
                    return Ok(lhs);
                }
            }
        }
        self.evaluate(right)
    }

    fn eval_func_call(&self, name: &str, args: &[Expr]) -> Result<Value, EvalError> {
        let evaluated: Result<Vec<Value>, EvalError> =
            args.iter().map(|arg| self.evaluate(arg)).collect();
        self.functions.call(name, &evaluated?)
    }
}

/// Returns `true` if a [`Value`] is considered truthy in GitHub Actions
/// expression semantics.
///
/// Falsy values: `Null`, `Bool(false)`, `Number(0)`, `String("")`.
/// Everything else is truthy.
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i != 0
            } else if let Some(f) = n.as_f64() {
                f != 0.0
            } else {
                true
            }
        }
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// Compares two [`Value`]s for ordering, following GitHub Actions coercion rules.
///
/// - Both numbers → numeric comparison (using `total_cmp`)
/// - Both strings → lexicographic comparison
/// - Both bools → `false < true`
/// - Otherwise → [`EvalError::TypeError`]
fn compare_values(left: &Value, right: &Value) -> Result<std::cmp::Ordering, EvalError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            let af = a
                .as_f64()
                .ok_or_else(|| EvalError::TypeError("left operand is not a valid number".into()))?;
            let bf = b.as_f64().ok_or_else(|| {
                EvalError::TypeError("right operand is not a valid number".into())
            })?;
            Ok(af.total_cmp(&bf))
        }
        (Value::String(a), Value::String(b)) => Ok(a.cmp(b)),
        (Value::Bool(a), Value::Bool(b)) => Ok(a.cmp(b)),
        _ => Err(EvalError::TypeError(
            "cannot compare values of different types".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn ctx() -> EvalContext {
        EvalContext::new()
    }

    fn eval(expr: &Expr) -> Result<Value, EvalError> {
        let c = ctx();
        Evaluator::new(&c).evaluate(expr)
    }

    #[test]
    fn eval_literal_bool_true() {
        let result = eval(&Expr::Literal(Literal::Bool(true))).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn eval_literal_bool_false() {
        let result = eval(&Expr::Literal(Literal::Bool(false))).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn eval_literal_int() {
        let result = eval(&Expr::Literal(Literal::Int(42))).unwrap();
        assert_eq!(result, json!(42));
    }

    #[test]
    fn eval_literal_int_negative() {
        let result = eval(&Expr::Literal(Literal::Int(-7))).unwrap();
        assert_eq!(result, json!(-7));
    }

    #[test]
    fn eval_literal_float() {
        let result = eval(&Expr::Literal(Literal::Float(std::f64::consts::PI))).unwrap();
        assert_eq!(result, json!(std::f64::consts::PI));
    }

    #[test]
    fn eval_literal_string() {
        let result = eval(&Expr::Literal(Literal::String("hello".into()))).unwrap();
        assert_eq!(result, json!("hello"));
    }

    #[test]
    fn eval_literal_null() {
        let result = eval(&Expr::Literal(Literal::Null)).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn eval_variable_stub() {
        let result = eval(&Expr::Variable("foo".into())).unwrap();
        assert_eq!(result, json!("${{ foo }}"));
    }

    #[test]
    fn eval_variable_stub_env() {
        let result = eval(&Expr::Variable("bar".into())).unwrap();
        assert_eq!(result, json!("${{ bar }}"));
    }

    #[test]
    fn eval_property_access_on_non_object() {
        let result = eval(&Expr::PropertyAccess(
            Box::new(Expr::Literal(Literal::String("hello".into()))),
            "length".into(),
        ));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EvalError::TypeError(_)));
    }

    #[test]
    fn eval_property_access_on_object() {
        let json_str = Expr::Literal(Literal::String(r#"{"x": 10}"#.into()));
        let from_json = Expr::FuncCall("fromJson".into(), vec![json_str]);
        let prop_access = Expr::PropertyAccess(Box::new(from_json), "x".into());
        let result = eval(&prop_access).unwrap();
        assert_eq!(result, json!(10));
    }

    #[test]
    fn eval_index_access_array() {
        let json_str = Expr::Literal(Literal::String(r#"["a", "b", "c"]"#.into()));
        let from_json = Expr::FuncCall("fromJson".into(), vec![json_str]);
        let idx = Expr::IndexAccess(
            Box::new(from_json),
            Box::new(Expr::Literal(Literal::Int(1))),
        );
        let result = eval(&idx).unwrap();
        assert_eq!(result, json!("b"));
    }

    #[test]
    fn eval_index_access_object() {
        let json_str = Expr::Literal(Literal::String(r#"{"key": "val"}"#.into()));
        let from_json = Expr::FuncCall("fromJson".into(), vec![json_str]);
        let idx = Expr::IndexAccess(
            Box::new(from_json),
            Box::new(Expr::Literal(Literal::String("key".into()))),
        );
        let result = eval(&idx).unwrap();
        assert_eq!(result, json!("val"));
    }

    #[test]
    fn eval_index_access_type_error() {
        let idx = Expr::IndexAccess(
            Box::new(Expr::Literal(Literal::String("hello".into()))),
            Box::new(Expr::Literal(Literal::Int(0))),
        );
        let result = eval(&idx);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EvalError::TypeError(_)));
    }

    #[test]
    fn eval_array_deref_stub() {
        let inner = Expr::Literal(Literal::Int(99));
        let deref = Expr::ArrayDeref(Box::new(inner));
        let result = eval(&deref).unwrap();
        assert_eq!(result, json!(99));
    }

    #[test]
    fn eval_not_true() {
        let expr = Expr::Not(Box::new(Expr::Literal(Literal::Bool(true))));
        let result = eval(&expr).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn eval_not_false() {
        let expr = Expr::Not(Box::new(Expr::Literal(Literal::Bool(false))));
        let result = eval(&expr).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn eval_not_empty_string() {
        let expr = Expr::Not(Box::new(Expr::Literal(Literal::String(String::new()))));
        let result = eval(&expr).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn eval_not_non_empty_string() {
        let expr = Expr::Not(Box::new(Expr::Literal(Literal::String("hi".into()))));
        let result = eval(&expr).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn eval_not_null() {
        let expr = Expr::Not(Box::new(Expr::Literal(Literal::Null)));
        let result = eval(&expr).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn eval_not_zero() {
        let expr = Expr::Not(Box::new(Expr::Literal(Literal::Int(0))));
        let result = eval(&expr).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn eval_compare_eq_numbers_true() {
        let expr = Expr::Compare(
            CompareOp::Eq,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_eq_numbers_false() {
        let expr = Expr::Compare(
            CompareOp::Eq,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(false));
    }

    #[test]
    fn eval_compare_neq_numbers() {
        let expr = Expr::Compare(
            CompareOp::Neq,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_lt_numbers_true() {
        let expr = Expr::Compare(
            CompareOp::Lt,
            Box::new(Expr::Literal(Literal::Int(2))),
            Box::new(Expr::Literal(Literal::Int(10))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_lt_numbers_false() {
        let expr = Expr::Compare(
            CompareOp::Lt,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(2))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(false));
    }

    #[test]
    fn eval_compare_gt_numbers() {
        let expr = Expr::Compare(
            CompareOp::Gt,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(2))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_lte_numbers_equal() {
        let expr = Expr::Compare(
            CompareOp::Lte,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_lte_numbers_less() {
        let expr = Expr::Compare(
            CompareOp::Lte,
            Box::new(Expr::Literal(Literal::Int(3))),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_gte_numbers() {
        let expr = Expr::Compare(
            CompareOp::Gte,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_strings_eq() {
        let expr = Expr::Compare(
            CompareOp::Eq,
            Box::new(Expr::Literal(Literal::String("abc".into()))),
            Box::new(Expr::Literal(Literal::String("abc".into()))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_strings_lt() {
        let expr = Expr::Compare(
            CompareOp::Lt,
            Box::new(Expr::Literal(Literal::String("abc".into()))),
            Box::new(Expr::Literal(Literal::String("xyz".into()))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_bools() {
        let expr = Expr::Compare(
            CompareOp::Lt,
            Box::new(Expr::Literal(Literal::Bool(false))),
            Box::new(Expr::Literal(Literal::Bool(true))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_compare_type_mismatch() {
        let expr = Expr::Compare(
            CompareOp::Eq,
            Box::new(Expr::Literal(Literal::Int(1))),
            Box::new(Expr::Literal(Literal::String("1".into()))),
        );
        let result = eval(&expr);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EvalError::TypeError(_)));
    }

    #[test]
    fn eval_logical_and_both_truthy() {
        let expr = Expr::Logical(
            LogicalOp::And,
            Box::new(Expr::Literal(Literal::Bool(true))),
            Box::new(Expr::Literal(Literal::Int(42))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(42));
    }

    #[test]
    fn eval_logical_and_short_circuit() {
        let expr = Expr::Logical(
            LogicalOp::And,
            Box::new(Expr::Literal(Literal::Bool(false))),
            Box::new(Expr::Literal(Literal::String("never".into()))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(false));
    }

    #[test]
    fn eval_logical_or_both_falsy() {
        let expr = Expr::Logical(
            LogicalOp::Or,
            Box::new(Expr::Literal(Literal::Bool(false))),
            Box::new(Expr::Literal(Literal::Int(0))),
        );
        assert_eq!(eval(&expr).unwrap(), json!(0));
    }

    #[test]
    fn eval_logical_or_short_circuit() {
        let expr = Expr::Logical(
            LogicalOp::Or,
            Box::new(Expr::Literal(Literal::String("first".into()))),
            Box::new(Expr::Literal(Literal::String("never".into()))),
        );
        assert_eq!(eval(&expr).unwrap(), json!("first"));
    }

    #[test]
    fn eval_func_call_contains_true() {
        let expr = Expr::FuncCall(
            "contains".into(),
            vec![
                Expr::Literal(Literal::String("Hello World".into())),
                Expr::Literal(Literal::String("world".into())),
            ],
        );
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_func_call_contains_false() {
        let expr = Expr::FuncCall(
            "contains".into(),
            vec![
                Expr::Literal(Literal::String("Hello World".into())),
                Expr::Literal(Literal::String("xyz".into())),
            ],
        );
        assert_eq!(eval(&expr).unwrap(), json!(false));
    }

    #[test]
    fn eval_func_call_unknown() {
        let expr = Expr::FuncCall("nonexistent".into(), vec![]);
        let result = eval(&expr);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EvalError::TypeError(_)));
    }

    #[test]
    fn eval_nested_logical_with_compare() {
        let left_cmp = Expr::Compare(
            CompareOp::Gt,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        let right_cmp = Expr::Compare(
            CompareOp::Lt,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(20))),
        );
        let expr = Expr::Logical(LogicalOp::And, Box::new(left_cmp), Box::new(right_cmp));
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn eval_nested_not_compare() {
        let cmp = Expr::Compare(
            CompareOp::Eq,
            Box::new(Expr::Literal(Literal::Int(1))),
            Box::new(Expr::Literal(Literal::Int(2))),
        );
        let expr = Expr::Not(Box::new(cmp));
        assert_eq!(eval(&expr).unwrap(), json!(true));
    }

    #[test]
    fn truthy_null_is_falsy() {
        assert!(!is_truthy(&Value::Null));
    }

    #[test]
    fn truthy_bool_false_is_falsy() {
        assert!(!is_truthy(&Value::Bool(false)));
    }

    #[test]
    fn truthy_bool_true_is_truthy() {
        assert!(is_truthy(&Value::Bool(true)));
    }

    #[test]
    fn truthy_zero_int_is_falsy() {
        assert!(!is_truthy(&json!(0)));
    }

    #[test]
    fn truthy_zero_float_is_falsy() {
        assert!(!is_truthy(&json!(0.0)));
    }

    #[test]
    fn truthy_non_zero_int_is_truthy() {
        assert!(is_truthy(&json!(42)));
    }

    #[test]
    fn truthy_non_zero_float_is_truthy() {
        assert!(is_truthy(&json!(std::f64::consts::PI)));
    }

    #[test]
    fn truthy_empty_string_is_falsy() {
        assert!(!is_truthy(&json!("")));
    }

    #[test]
    fn truthy_non_empty_string_is_truthy() {
        assert!(is_truthy(&json!("hello")));
    }

    #[test]
    fn truthy_empty_array_is_truthy() {
        assert!(is_truthy(&json!([])));
    }

    #[test]
    fn truthy_empty_object_is_truthy() {
        assert!(is_truthy(&json!({})));
    }
}
