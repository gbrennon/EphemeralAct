use std::fmt;

/// A named secret made available to a workflow run.
///
/// The value backs `${{ secrets.<name> }}` expressions and is kept out of
/// [`Debug`] output so run summaries and logs never leak it.
pub struct Secret {
    name: String,
    value: String,
}

impl Secret {
    /// Creates a secret from its name and value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephact::core::value_objects::Secret;
    /// let secret = Secret::new("CRATES_IO_TOKEN".into(), "my-token".into());
    /// assert_eq!(secret.name(), "CRATES_IO_TOKEN");
    /// assert_eq!(secret.value(), "my-token");
    /// ```
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }

    /// Returns the name the workflow references the secret by.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the raw secret value.
    ///
    /// Use with caution - this exposes the unredacted secret.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for Secret {
    /// Shows the secret name but redacts its value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret({}=***)", self.name)
    }
}

impl Clone for Secret {
    /// Clones the secret, preserving name and value.
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

impl PartialEq for Secret {
    /// Compares secrets by name and value.
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

impl Eq for Secret {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_value_but_keeps_name() {
        let secret = Secret::new("TOKEN".into(), "my-token".into());

        let debug = format!("{secret:?}");

        assert!(!debug.contains("my-token"));
        assert!(debug.contains("TOKEN"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn accessors_return_name_and_value() {
        let secret = Secret::new("TOKEN".into(), "my-token".into());

        assert_eq!(secret.name(), "TOKEN");
        assert_eq!(secret.value(), "my-token");
    }

    #[test]
    fn clone_preserves_name_and_value() {
        let secret = Secret::new("TOKEN".into(), "my-secret".into());

        let cloned = secret.clone();

        assert_eq!(cloned.name(), "TOKEN");
        assert_eq!(cloned.value(), "my-secret");
    }

    #[test]
    fn secrets_with_same_name_and_value_are_equal() {
        assert_eq!(
            Secret::new("TOKEN".into(), "x".into()),
            Secret::new("TOKEN".into(), "x".into())
        );
    }

    #[test]
    fn secrets_with_different_values_are_not_equal() {
        assert_ne!(
            Secret::new("TOKEN".into(), "x".into()),
            Secret::new("TOKEN".into(), "y".into())
        );
    }

    #[test]
    fn secrets_with_different_names_are_not_equal() {
        assert_ne!(
            Secret::new("A".into(), "x".into()),
            Secret::new("B".into(), "x".into())
        );
    }
}
