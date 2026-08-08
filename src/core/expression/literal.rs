use std::fmt;

/// Literal value types in GitHub Actions expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Bool(bool),
    Null,
    Int(i64),
    Float(f64),
    String(String),
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
