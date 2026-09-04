use crate::application::dtos::PullJobImageRequest;

/// Inbound port for pulling the image a job runs in.
pub trait PullJobImagePort: Send + Sync {
    /// Pulls the image the job's runner label maps to and returns it.
    fn execute(
        &self,
        request: PullJobImageRequest<'_>,
    ) -> Result<String, Box<dyn std::error::Error>>;
}
