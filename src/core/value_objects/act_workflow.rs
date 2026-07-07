#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActWorkflow(String);

impl ActWorkflow {
    /// Creates a new workflow from a path string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::ActWorkflow;
    /// let wf = ActWorkflow::new(".github/workflows/ci.yml".into());
    /// assert_eq!(wf.as_str(), ".github/workflows/ci.yml");
    /// ```
    pub fn new(path: String) -> Self {
        Self(path)
    }

    /// Returns the workflow path as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_path() {
        let wf = ActWorkflow::new(".github/workflows/ci.yml".into());
        assert_eq!(wf.as_str(), ".github/workflows/ci.yml");
    }
}
