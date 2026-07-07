#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupPolicy {
    CleanupOnExit,
    Preserve,
}

impl CleanupPolicy {
    /// Returns `true` if the ephemeral repository should be cleaned up after execution.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::CleanupPolicy;
    /// assert!(CleanupPolicy::CleanupOnExit.should_cleanup());
    /// assert!(!CleanupPolicy::Preserve.should_cleanup());
    /// ```
    pub fn should_cleanup(&self) -> bool {
        matches!(self, CleanupPolicy::CleanupOnExit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_on_exit_should_cleanup() {
        assert!(CleanupPolicy::CleanupOnExit.should_cleanup());
    }

    #[test]
    fn preserve_should_not_cleanup() {
        assert!(!CleanupPolicy::Preserve.should_cleanup());
    }
}
