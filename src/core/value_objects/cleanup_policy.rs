#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupPolicy {
    CleanupOnExit,
    Preserve,
}

impl CleanupPolicy {
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