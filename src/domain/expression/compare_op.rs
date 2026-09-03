use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_eq() {
        assert_eq!(CompareOp::Eq.to_string(), "==");
    }

    #[test]
    fn test_display_neq() {
        assert_eq!(CompareOp::Neq.to_string(), "!=");
    }

    #[test]
    fn test_display_lt() {
        assert_eq!(CompareOp::Lt.to_string(), "<");
    }

    #[test]
    fn test_display_lte() {
        assert_eq!(CompareOp::Lte.to_string(), "<=");
    }

    #[test]
    fn test_display_gt() {
        assert_eq!(CompareOp::Gt.to_string(), ">");
    }

    #[test]
    fn test_display_gte() {
        assert_eq!(CompareOp::Gte.to_string(), ">=");
    }
}
