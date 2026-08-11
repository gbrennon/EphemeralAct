use ephemeral_act::{core::ports::outbound::ImageMapper, infrastructure::PlatformImageMapper};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_platform_ubuntu_latest_returns_act_latest() {
        assert_eq!(
            PlatformImageMapper.map("ubuntu-latest"),
            "catthehacker/ubuntu:act-latest"
        );
    }

    #[test]
    fn map_platform_ubuntu_24_04_returns_act_24_04() {
        assert_eq!(
            PlatformImageMapper.map("ubuntu-24.04"),
            "catthehacker/ubuntu:act-24.04"
        );
    }

    #[test]
    fn map_platform_ubuntu_22_04_returns_act_22_04() {
        assert_eq!(
            PlatformImageMapper.map("ubuntu-22.04"),
            "catthehacker/ubuntu:act-22.04"
        );
    }

    #[test]
    fn map_platform_ubuntu_20_04_returns_act_20_04() {
        assert_eq!(
            PlatformImageMapper.map("ubuntu-20.04"),
            "catthehacker/ubuntu:act-20.04"
        );
    }

    #[test]
    fn map_platform_unknown_returns_input_unchanged() {
        assert_eq!(
            PlatformImageMapper.map("custom-image:latest"),
            "custom-image:latest"
        );
    }

    #[test]
    fn map_platform_empty_string_returns_empty() {
        assert_eq!(PlatformImageMapper.map(""), "");
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
    fn fallback_returns_ubuntu_latest() {
        assert_eq!(
            PlatformImageMapper.fallback(),
            "catthehacker/ubuntu:act-latest"
        );
    }
}
