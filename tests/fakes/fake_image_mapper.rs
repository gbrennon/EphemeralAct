use ephemeral_act::core::ports::outbound::ImageMapper;

#[allow(dead_code)]
pub struct FakeImageMapper;

impl ImageMapper for FakeImageMapper {
    fn map(&self, platform: &str) -> String {
        platform.to_string()
    }
    fn fallback(&self) -> String {
        "fake-image:latest".into()
    }
}
