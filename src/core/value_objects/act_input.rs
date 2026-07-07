#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActInput {
    key: String,
    value: String,
}

impl ActInput {
    /// Creates a new input with a key and value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::ActInput;
    /// let input = ActInput::new("environment".into(), "staging".into());
    /// assert_eq!(input.key(), "environment");
    /// assert_eq!(input.value(), "staging");
    /// ```
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    /// Returns the input key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the input value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_key_and_value() {
        let input = ActInput::new("environment".into(), "staging".into());
        assert_eq!(input.key(), "environment");
        assert_eq!(input.value(), "staging");
    }
}
