#![allow(dead_code)]
use ephact::infrastructure::images::ImageMapperPort;

pub struct FakeImageMapper;

impl ImageMapperPort for FakeImageMapper {
    fn map(&self, platform: &str) -> String {
        platform.to_string()
    }
    fn fallback(&self) -> String {
        "fake-image:latest".into()
    }

    fn clone_box(&self) -> Box<dyn ImageMapperPort> {
        Box::new(Self)
    }
}
