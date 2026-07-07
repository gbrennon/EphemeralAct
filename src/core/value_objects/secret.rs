use std::fmt;

pub struct Secret(String);

impl Secret {
    /// Creates a new secret from a raw value string.
    ///
    /// The value is stored as-is but its [`Debug`] output is redacted.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::Secret;
    /// let secret = Secret::new("my-token".into());
    /// assert_eq!(secret.as_str(), "my-token");
    /// ```
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the raw secret value.
    ///
    /// Use with caution — this exposes the unredacted secret.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    /// Redacts the secret value, showing `***` instead.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

impl Clone for Secret {
    /// Clones the secret, preserving the inner value.
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl PartialEq for Secret {
    /// Compares secrets by their inner value.
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Secret {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_value() {
        let secret = Secret::new("my-token".into());
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("my-token"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn as_str_returns_value() {
        let secret = Secret::new("my-token".into());
        assert_eq!(secret.as_str(), "my-token");
    }
}
