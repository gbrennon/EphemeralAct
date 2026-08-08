use serde::Deserialize;

use super::Matrix;

/// A matrix strategy for generating multiple job runs.
///
/// Each combination of matrix variables produces a separate job run.
/// Supports `include`/`exclude` for fine-grained control.
///
/// # Examples
///
/// ```
/// use ephemeral_act::core::workflow::Strategy;
///
/// let yaml = r#"
/// matrix:
///   os: [ubuntu-latest, macos-latest]
///   rust: [stable, nightly]
/// "#;
/// let strategy: Strategy = serde_yaml::from_str(yaml).unwrap();
/// assert_eq!(strategy.matrix.as_ref().unwrap().variables.len(), 2);
/// ```
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Strategy {
    /// The matrix of variables to expand.
    #[serde(default)]
    pub matrix: Option<Matrix>,

    /// Whether to cancel all in-progress jobs if any matrix job fails.
    #[serde(rename = "fail-fast")]
    #[serde(default = "default_fail_fast")]
    pub fail_fast: bool,

    /// Maximum number of jobs to run in parallel.
    #[serde(rename = "max-parallel")]
    #[serde(default)]
    pub max_parallel: Option<usize>,
}

fn default_fail_fast() -> bool {
    true
}

impl Strategy {
    /// Returns `true` if this strategy has a matrix defined.
    pub fn has_matrix(&self) -> bool {
        self.matrix.is_some()
    }

    /// Returns the number of matrix combinations (before include/exclude).
    pub fn combination_count(&self) -> usize {
        self.matrix
            .as_ref()
            .map(|m| {
                if m.variables.is_empty() {
                    0
                } else {
                    m.variables.values().map(|v| v.len()).product()
                }
            })
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_matrix_strategy() {
        let yaml = r#"
matrix:
  os: [ubuntu-latest, macos-latest]
  rust: [stable, nightly]
fail-fast: false
max-parallel: 2
"#;
        let strategy: Strategy = serde_yaml::from_str(yaml).unwrap();
        assert!(strategy.has_matrix());
        assert!(!strategy.fail_fast);
        assert_eq!(strategy.max_parallel, Some(2));
        assert_eq!(strategy.combination_count(), 4); // 2 os × 2 rust
    }

    #[test]
    fn parse_strategy_with_include_exclude() {
        let yaml = r#"
matrix:
  os: [ubuntu-latest]
  rust: [stable]
  include:
    - os: macos-latest
      rust: nightly
  exclude:
    - os: ubuntu-latest
      rust: stable
"#;
        let strategy: Strategy = serde_yaml::from_str(yaml).unwrap();
        let matrix = strategy.matrix.unwrap();
        assert_eq!(matrix.include.len(), 1);
        assert_eq!(matrix.exclude.len(), 1);
    }

    #[test]
    fn parse_strategy_defaults() {
        let yaml = "matrix:\n  os: [ubuntu-latest]\n";
        let strategy: Strategy = serde_yaml::from_str(yaml).unwrap();
        assert!(strategy.fail_fast); // default
        assert_eq!(strategy.max_parallel, None);
    }

    #[test]
    fn combination_count_empty_matrix() {
        let yaml = "matrix: {}\n";
        let strategy: Strategy = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(strategy.combination_count(), 0);
    }

    #[test]
    fn combination_count_no_matrix() {
        let strategy = Strategy {
            matrix: None,
            fail_fast: true,
            max_parallel: None,
        };
        assert_eq!(strategy.combination_count(), 1);
    }
}
