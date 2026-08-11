/// Outbound port for mapping CI platform labels to container images.
///
/// Adapters implement this trait to provide platform-specific image
/// resolution. The [`fallback`](ImageMapper::fallback) method returns a
/// default image used when the primary image cannot be pulled.
pub trait ImageMapper {
    /// Maps a CI platform label (e.g. `ubuntu-latest`, `codeberg-tiny`)
    /// to a container image name.
    fn map(&self, platform: &str) -> String;

    /// Returns the fallback image used when the primary image is not
    /// available.
    fn fallback(&self) -> String;
}
