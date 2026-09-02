use crate::core::{errors::ActionError, value_objects::RemoteActionReference};

/// Host assumed when a reference omits scheme and host, matching the shorthand
/// `owner/repo@ref` form used by GitHub workflows.
const DEFAULT_HOST: &str = "github.com";

/// Revision used when a reference carries no `@ref` suffix.
const DEFAULT_GIT_REF: &str = "main";

/// A parsed `uses:` value, classified by how the action must be resolved.
///
/// Parsing is platform-agnostic: shorthand references resolve against
/// `github.com`, while any forge can be addressed with an explicit URL, so
/// `actions/cache@v4` and `https://data.forgejo.org/actions/cache@v4` are both
/// understood.
///
/// # Examples
///
/// ```
/// # use ephemeral_act::core::value_objects::ActionReference;
/// let reference = ActionReference::parse("https://data.forgejo.org/actions/cache@v4").unwrap();
/// let ActionReference::Remote(remote) = reference else { panic!("expected a remote action") };
/// assert_eq!(remote.clone_url(), "https://data.forgejo.org/actions/cache");
/// assert_eq!(remote.git_ref(), "v4");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionReference {
    /// An action stored in the repository under test, addressed by a relative
    /// path (`./.forgejo/actions/publish`).
    Local(String),

    /// An action published in a git repository on some forge.
    Remote(RemoteActionReference),

    /// An action delivered as a container image (`docker://image:tag`).
    Docker(String),
}

impl ActionReference {
    /// Classifies a raw `uses:` value.
    ///
    /// # Errors
    ///
    /// Returns [`ActionError::InvalidReference`] when the value names neither a
    /// relative path, a container image, nor an `owner/repo` pair.
    pub fn parse(raw: &str) -> Result<Self, ActionError> {
        let reference = raw.trim();
        let invalid = || ActionError::InvalidReference(raw.to_string());

        if reference.is_empty() {
            return Err(invalid());
        }
        if let Some(image) = reference.strip_prefix("docker://") {
            return Ok(Self::Docker(image.to_string()));
        }
        if reference.starts_with("./") || reference.starts_with("../") {
            return Ok(Self::Local(reference.to_string()));
        }

        let (location, git_ref) = match reference.rsplit_once('@') {
            Some((location, git_ref)) if !location.is_empty() && !git_ref.is_empty() => {
                (location, git_ref)
            }
            Some(_) => return Err(invalid()),
            None => (reference, DEFAULT_GIT_REF),
        };

        let (scheme, host, path) = match location.split_once("://") {
            Some((scheme, remainder)) => {
                let (host, path) = remainder.split_once('/').ok_or_else(invalid)?;
                if scheme.is_empty() || host.is_empty() {
                    return Err(invalid());
                }
                (scheme.to_string(), host.to_string(), path)
            }
            None => ("https".to_string(), DEFAULT_HOST.to_string(), location),
        };

        let mut segments = path.split('/').filter(|segment| !segment.is_empty());
        let owner = segments.next().ok_or_else(invalid)?;
        let repo = segments.next().ok_or_else(invalid)?;
        let directory: Vec<&str> = segments.collect();

        Ok(Self::Remote(RemoteActionReference::new(
            scheme,
            host,
            owner.to_string(),
            repo.to_string(),
            (!directory.is_empty()).then(|| directory.join("/")),
            git_ref.to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote(raw: &str) -> RemoteActionReference {
        match ActionReference::parse(raw).unwrap() {
            ActionReference::Remote(remote) => remote,
            other => panic!("expected a remote action, got {other:?}"),
        }
    }

    #[test]
    fn parse_relative_path_is_local() {
        assert_eq!(
            ActionReference::parse("./.forgejo/actions/publish").unwrap(),
            ActionReference::Local("./.forgejo/actions/publish".into())
        );
    }

    #[test]
    fn parse_parent_relative_path_is_local() {
        assert_eq!(
            ActionReference::parse("../shared/action").unwrap(),
            ActionReference::Local("../shared/action".into())
        );
    }

    #[test]
    fn parse_docker_image_is_docker() {
        assert_eq!(
            ActionReference::parse("docker://alpine:3.20").unwrap(),
            ActionReference::Docker("alpine:3.20".into())
        );
    }

    #[test]
    fn parse_shorthand_defaults_to_github() {
        let reference = remote("actions/checkout@v4");

        assert_eq!(reference.clone_url(), "https://github.com/actions/checkout");
        assert_eq!(reference.git_ref(), "v4");
        assert_eq!(reference.directory(), None);
    }

    #[test]
    fn parse_shorthand_without_ref_defaults_to_main() {
        assert_eq!(remote("actions/checkout").git_ref(), "main");
    }

    #[test]
    fn parse_full_url_keeps_host() {
        let reference = remote("https://data.forgejo.org/actions/cache@v4");

        assert_eq!(reference.host(), "data.forgejo.org");
        assert_eq!(reference.owner(), "actions");
        assert_eq!(reference.repo(), "cache");
        assert_eq!(reference.git_ref(), "v4");
    }

    #[test]
    fn parse_keeps_action_subdirectory() {
        let reference = remote("https://gitlab.com/group/tools/deploy/action@main");

        assert_eq!(reference.clone_url(), "https://gitlab.com/group/tools");
        assert_eq!(reference.directory(), Some("deploy/action"));
    }

    #[test]
    fn parse_commit_sha_ref() {
        assert_eq!(
            remote("actions/cache@a1b2c3d4e5f6").git_ref(),
            "a1b2c3d4e5f6"
        );
    }

    #[test]
    fn parse_rejects_empty_reference() {
        assert_eq!(
            ActionReference::parse("   "),
            Err(ActionError::InvalidReference("   ".into()))
        );
    }

    #[test]
    fn parse_rejects_reference_without_repository() {
        assert_eq!(
            ActionReference::parse("checkout@v4"),
            Err(ActionError::InvalidReference("checkout@v4".into()))
        );
    }

    #[test]
    fn parse_rejects_reference_with_empty_ref() {
        assert_eq!(
            ActionReference::parse("actions/cache@"),
            Err(ActionError::InvalidReference("actions/cache@".into()))
        );
    }

    #[test]
    fn parse_rejects_url_without_path() {
        assert_eq!(
            ActionReference::parse("https://data.forgejo.org@v4"),
            Err(ActionError::InvalidReference(
                "https://data.forgejo.org@v4".into()
            ))
        );
    }
}
