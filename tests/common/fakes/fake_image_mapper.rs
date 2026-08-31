#![allow(dead_code)]
use ephemeral_act::core::ports::outbound::ImageMapperPort;

pub struct FakeImageMapper;

impl ImageMapperPort for FakeImageMapper {
    fn map(&self, platform: &str) -> String {
        platform.to_string()
    }
    fn fallback(&self) -> String {
        "fake-image:latest".into()
    }
}
