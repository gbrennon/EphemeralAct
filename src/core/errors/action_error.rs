use std::fmt;

/// Errors raised while resolving, fetching, or executing an action reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    /// The `uses:` value does not name an action that can be resolved.
    InvalidReference(String),

    /// The action repository could not be retrieved from its host.
    FetchFailed(String),

    /// The action was resolved but declares an execution model that this
    /// runner does not implement.
    Unsupported(String),
}

impl fmt::Display for ActionError {
    /// Renders the error as a single line suitable for a step summary.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidReference(reference) => {
                write!(f, "invalid action reference: {reference}")
            }
            Self::FetchFailed(details) => write!(f, "failed to fetch action: {details}"),
            Self::Unsupported(details) => write!(f, "unsupported action: {details}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_invalid_reference_names_the_reference() {
        let error = ActionError::InvalidReference("cache".into());

        assert_eq!(error.to_string(), "invalid action reference: cache");
    }

    #[test]
    fn display_fetch_failed_includes_details() {
        let error = ActionError::FetchFailed("git clone exited with 128".into());

        assert_eq!(
            error.to_string(),
            "failed to fetch action: git clone exited with 128"
        );
    }

    #[test]
    fn display_unsupported_includes_details() {
        let error = ActionError::Unsupported("docker action".into());

        assert_eq!(error.to_string(), "unsupported action: docker action");
    }
}
