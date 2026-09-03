/// Request DTO for the
/// [`PullJobImagePort`](crate::application::ports::inbound::pull_job_image_port::PullJobImagePort)
/// inbound port.
pub struct PullJobImageRequest<'a> {
    /// Runner label the job declared, when it declared one.
    pub runs_on: Option<&'a str>,
}
