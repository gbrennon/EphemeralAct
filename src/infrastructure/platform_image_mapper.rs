use crate::core::ports::outbound::ImageMapper;

/// Maps CI platform `runs-on` labels to container image names.
///
/// Known GitHub-hosted labels resolve to `catthehacker/ubuntu` images
/// designed for local workflow execution. Forgejo/Codeberg runner labels
/// (`codeberg-*`) map to the default Ubuntu image. Unknown platforms are
/// returned as-is (assumed to be user-provided image names).
///
/// # Examples
///
/// ```
/// use ephemeral_act::core::ports::outbound::ImageMapper;
/// use ephemeral_act::infrastructure::PlatformImageMapper;
///
/// let mapper = PlatformImageMapper;
/// assert_eq!(mapper.map("ubuntu-latest"), "catthehacker/ubuntu:act-latest");
/// assert_eq!(mapper.map("codeberg-tiny"), "catthehacker/ubuntu:act-latest");
/// assert_eq!(mapper.map("my-custom-image"), "my-custom-image");
/// assert_eq!(mapper.fallback(), "catthehacker/ubuntu:act-latest");
/// ```
pub struct PlatformImageMapper;

impl ImageMapper for PlatformImageMapper {
    fn map(&self, platform: &str) -> String {
        match platform {
            "ubuntu-latest" => "catthehacker/ubuntu:act-latest",
            "ubuntu-24.04" => "catthehacker/ubuntu:act-24.04",
            "ubuntu-22.04" => "catthehacker/ubuntu:act-22.04",
            "ubuntu-20.04" => "catthehacker/ubuntu:act-20.04",
            p if p.starts_with("codeberg-") => "catthehacker/ubuntu:act-latest",
            other => other,
        }
        .to_string()
    }

    fn fallback(&self) -> String {
        "catthehacker/ubuntu:act-latest".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_ubuntu_latest() {
        assert_eq!(
            PlatformImageMapper.map("ubuntu-latest"),
            "catthehacker/ubuntu:act-latest"
        );
    }

    #[test]
    fn maps_ubuntu_2404() {
        assert_eq!(
            PlatformImageMapper.map("ubuntu-24.04"),
            "catthehacker/ubuntu:act-24.04"
        );
    }

    #[test]
    fn maps_ubuntu_2204() {
        assert_eq!(
            PlatformImageMapper.map("ubuntu-22.04"),
            "catthehacker/ubuntu:act-22.04"
        );
    }

    #[test]
    fn maps_ubuntu_2004() {
        assert_eq!(
            PlatformImageMapper.map("ubuntu-20.04"),
            "catthehacker/ubuntu:act-20.04"
        );
    }

    #[test]
    fn maps_codeberg_tiny_to_ubuntu() {
        assert_eq!(
            PlatformImageMapper.map("codeberg-tiny"),
            "catthehacker/ubuntu:act-latest"
        );
    }

    #[test]
    fn maps_codeberg_medium_to_ubuntu() {
        assert_eq!(
            PlatformImageMapper.map("codeberg-medium"),
            "catthehacker/ubuntu:act-latest"
        );
    }

    #[test]
    fn maps_codeberg_large_to_ubuntu() {
        assert_eq!(
            PlatformImageMapper.map("codeberg-large"),
            "catthehacker/ubuntu:act-latest"
        );
    }

    #[test]
    fn passes_unknown_platform_through() {
        assert_eq!(PlatformImageMapper.map("windows-latest"), "windows-latest");
    }

    #[test]
    fn passes_custom_image_through() {
        assert_eq!(
            PlatformImageMapper.map("custom-image:latest"),
            "custom-image:latest"
        );
    }

    #[test]
    fn passes_empty_string_through() {
        assert_eq!(PlatformImageMapper.map(""), "");
    }

    #[test]
    fn fallback_returns_ubuntu_latest() {
        assert_eq!(
            PlatformImageMapper.fallback(),
            "catthehacker/ubuntu:act-latest"
        );
    }
}
