use crate::application::ports::outbound::ImageMapperPort;

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
/// use ephact::application::ports::outbound::ImageMapperPort;
/// use ephact::infrastructure::PlatformImageMapper;
///
/// let mapper = PlatformImageMapper;
/// assert_eq!(mapper.map("ubuntu-latest"), "catthehacker/ubuntu:act-latest");
/// assert_eq!(mapper.map("codeberg-tiny"), "catthehacker/ubuntu:act-latest");
/// assert_eq!(mapper.map("my-custom-image"), "my-custom-image");
/// assert_eq!(mapper.fallback(), "catthehacker/ubuntu:act-latest");
/// ```
pub struct PlatformImageMapper;

impl ImageMapperPort for PlatformImageMapper {
    fn map(&self, platform: &str) -> String {
        match platform {
            "ubuntu-latest" => "catthehacker/ubuntu:act-latest",
            "ubuntu-24.04" => "catthehacker/ubuntu:act-24.04",
            "ubuntu-22.04" => "catthehacker/ubuntu:act-22.04",
            "ubuntu-20.04" => "catthehacker/ubuntu:act-20.04",
            "docker" => "catthehacker/ubuntu:act-latest",
            p if p.starts_with("codeberg-") => "catthehacker/ubuntu:act-latest",
            other => other,
        }
        .to_string()
    }

    fn fallback(&self) -> String {
        "catthehacker/ubuntu:act-latest".to_string()
    }
}
