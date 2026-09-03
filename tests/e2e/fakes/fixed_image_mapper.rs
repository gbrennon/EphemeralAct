use ephact::application::ports::outbound::ImageMapperPort;

/// Image every runner label maps to during an end-to-end scenario.
pub const RUNNER_IMAGE: &str = "e2e-runner:latest";

/// Image mapper that resolves every runner label to a single image, so a
/// scenario can count the containers a workflow created.
pub struct FixedImageMapper;

impl ImageMapperPort for FixedImageMapper {
    fn map(&self, _platform: &str) -> String {
        RUNNER_IMAGE.into()
    }

    fn fallback(&self) -> String {
        RUNNER_IMAGE.into()
    }
}
