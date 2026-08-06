/// AST node types for GitHub Actions `${{ }}` expressions.
///
/// Represents the full expression language: literals, context access,
/// property/index dereferencing, comparisons, logical operators,
/// function calls, and the ternary-like `a && b || c` pattern.
use std::fmt;

/// A complete expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal value: string, number, boolean, or null.
    Literal(Literal),
    /// A top-level context variable: `github`, `env`, `job`, `steps`, etc.
    Variable(String),
    /// Property access: `foo.bar`
    PropertyAccess(Box<Expr>, String),
    /// Index access: `foo[bar]` — string key or numeric index.
    IndexAccess(Box<Expr>, Box<Expr>),
    /// Array dereference: `foo.*` — flattens array of objects.
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

/// Literal value types in GitHub Actions expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Bool(bool),
    Null,
    Int(i64),
    Float(f64),
    String(String),
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
}

/// Logical operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

impl fmt::Display for CompareOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompareOp::Eq => write!(f, "=="),
            CompareOp::Neq => write!(f, "!="),
            CompareOp::Lt => write!(f, "<"),
            CompareOp::Lte => write!(f, "<="),
            CompareOp::Gt => write!(f, ">"),
            CompareOp::Gte => write!(f, ">="),
        }
    }
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOp::And => write!(f, "&&"),
            LogicalOp::Or => write!(f, "||"),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Null => write!(f, "null"),
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "'{}'", s),
        }
    }
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
}
