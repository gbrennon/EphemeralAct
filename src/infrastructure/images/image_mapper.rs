/// Outbound port for mapping CI platform labels to container images.
///
/// The [`fallback`](ImageMapperPort::fallback) method returns a default
/// image used when the primary image cannot be pulled.
pub trait ImageMapperPort: Send + Sync {
    /// Map a runner label (e.g. `ubuntu-latest`) to a container image tag.
    fn map(&self, platform: &str) -> String;

    /// Image used when the mapped image fails to pull.
    fn fallback(&self) -> String;

    /// Clone the trait object into a Box.
    fn clone_box(&self) -> Box<dyn ImageMapperPort>;
}

impl Clone for Box<dyn ImageMapperPort> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
