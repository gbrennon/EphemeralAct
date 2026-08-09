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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_bool_true() {
        assert_eq!(Literal::Bool(true).to_string(), "true");
    }

    #[test]
    fn display_bool_false() {
        assert_eq!(Literal::Bool(false).to_string(), "false");
    }

    #[test]
    fn display_null() {
        assert_eq!(Literal::Null.to_string(), "null");
    }

    #[test]
    fn display_int() {
        assert_eq!(Literal::Int(42).to_string(), "42");
    }

    #[test]
    fn display_float() {
        assert_eq!(Literal::Float(3.14).to_string(), "3.14");
    }

    #[test]
    fn display_string() {
        assert_eq!(Literal::String("hello".into()).to_string(), "'hello'");
    }
}
