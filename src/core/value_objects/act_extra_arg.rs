#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActExtraArg(String);

impl ActExtraArg {
    /// Creates a new extra argument from its string value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::ActExtraArg;
    /// let arg = ActExtraArg::new("--verbose".into());
    /// assert_eq!(arg.as_str(), "--verbose");
    /// ```
    pub fn new(arg: String) -> Self {
        Self(arg)
    }

    /// Returns the argument string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_arg() {
        let arg = ActExtraArg::new("--verbose".into());
        assert_eq!(arg.as_str(), "--verbose");
    }
}
