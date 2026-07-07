use std::fmt;

use crate::core::errors::CoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryName(pub String);

impl RepositoryName {
    pub fn new(name: String) -> Result<Self, CoreError> {
        if name.is_empty() {
            Err(CoreError::EmptyRepositoryName)
        } else {
            Ok(Self(name))
        }
    }
}

impl fmt::Display for RepositoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_valid_name_succeeds() {
        let name = RepositoryName::new("my-repo".into()).unwrap();
        assert_eq!(name.0, "my-repo");
    }

    #[test]
    fn new_with_empty_name_returns_empty_repository_name() {
        let result = RepositoryName::new("".into());
        assert_eq!(result, Err(CoreError::EmptyRepositoryName));
    }

    #[test]
    fn display_formats_inner_string() {
        let name = RepositoryName::new("my-repo".into()).unwrap();
        assert_eq!(format!("{}", name), "my-repo");
    }
}