use ephemeral_act::core::ports::outbound::ImageMapper;

/// Stub image mapper that passes platforms through unchanged.
pub(super) struct StubImageMapper;

impl ImageMapper for StubImageMapper {
    fn map(&self, platform: &str) -> String {
        platform.to_string()
    }

    fn fallback(&self) -> String {
        "catthehacker/ubuntu:act-latest".into()
    }
}
