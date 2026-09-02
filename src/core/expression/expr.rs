/// AST node types for GitHub Actions `${{ }}` expressions.
///
/// Represents the full expression language: literals, context access,
/// property/index dereferencing, comparisons, logical operators,
/// function calls, and the ternary-like `a && b || c` pattern.
use std::fmt;

use super::{compare_op::CompareOp, literal::Literal, logical_op::LogicalOp};

/// A complete expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal value: string, number, boolean, or null.
    Literal(Literal),
    /// A top-level context variable: `github`, `env`, `job`, `steps`, etc.
    Variable(String),
    /// Property access: `foo.bar`
    PropertyAccess(Box<Expr>, String),
    /// Index access: `foo[bar]` - string key or numeric index.
    IndexAccess(Box<Expr>, Box<Expr>),
    /// Array dereference: `foo.*` - flattens array of objects.
    ArrayDeref(Box<Expr>),
    /// Logical NOT: `!expr`
    Not(Box<Expr>),
    /// Comparison: `a == b`, `a != b`, `a < b`, etc.
    Compare(CompareOp, Box<Expr>, Box<Expr>),
    /// Logical AND/OR: `a && b`, `a || b`
    Logical(LogicalOp, Box<Expr>, Box<Expr>),
    /// Function call: `contains(search, item)`
    FuncCall(String, Vec<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::PropertyAccess(obj, prop) => write!(f, "{}.{}", obj, prop),
            Expr::IndexAccess(obj, idx) => write!(f, "{}[{}]", obj, idx),
            Expr::ArrayDeref(obj) => write!(f, "{}.*", obj),
            Expr::Not(expr) => write!(f, "!{}", expr),
            Expr::Compare(op, lhs, rhs) => write!(f, "{} {} {}", lhs, op, rhs),
            Expr::Logical(op, lhs, rhs) => write!(f, "{} {} {}", lhs, op, rhs),
            Expr::FuncCall(name, args) => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_literal_bool() {
        assert_eq!(Literal::Bool(true).to_string(), "true");
        assert_eq!(Literal::Bool(false).to_string(), "false");
    }

    #[test]
    fn display_literal_null() {
        assert_eq!(Literal::Null.to_string(), "null");
    }

    #[test]
    fn display_literal_int() {
        assert_eq!(Literal::Int(42).to_string(), "42");
    }

    #[test]
    fn display_literal_string() {
        assert_eq!(Literal::String("hello".into()).to_string(), "'hello'");
    }

    #[test]
    fn display_variable() {
        assert_eq!(Expr::Variable("github".into()).to_string(), "github");
    }

    #[test]
    fn display_property_access() {
        let expr = Expr::PropertyAccess(
            Box::new(Expr::Variable("github".into())),
            "event_name".into(),
        );
        assert_eq!(expr.to_string(), "github.event_name");
    }

    #[test]
    fn display_func_call() {
        let expr = Expr::FuncCall(
            "contains".into(),
            vec![
                Expr::Literal(Literal::String("hello".into())),
                Expr::Literal(Literal::String("ll".into())),
            ],
        );
        assert_eq!(expr.to_string(), "contains('hello', 'll')");
    }

    #[test]
    fn display_compare() {
        let expr = Expr::Compare(
            CompareOp::Eq,
            Box::new(Expr::Variable("github".into())),
            Box::new(Expr::Literal(Literal::String("push".into()))),
        );
        assert_eq!(expr.to_string(), "github == 'push'");
    }

    #[test]
    fn display_logical() {
        let expr = Expr::Logical(
            LogicalOp::And,
            Box::new(Expr::Literal(Literal::Bool(true))),
            Box::new(Expr::Literal(Literal::Bool(false))),
        );
        assert_eq!(expr.to_string(), "true && false");
    }

    #[test]
    fn display_index_access() {
        let expr = Expr::IndexAccess(
            Box::new(Expr::Variable("arr".into())),
            Box::new(Expr::Literal(Literal::Int(0))),
        );
        assert_eq!(expr.to_string(), "arr[0]");
    }

    #[test]
    fn display_array_deref() {
        let expr = Expr::ArrayDeref(Box::new(Expr::Variable("arr".into())));
        assert_eq!(expr.to_string(), "arr.*");
    }

    #[test]
    fn display_not() {
        let expr = Expr::Not(Box::new(Expr::Variable("flag".into())));
        assert_eq!(expr.to_string(), "!flag");
    }
}
