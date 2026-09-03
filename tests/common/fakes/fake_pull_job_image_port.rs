#![allow(dead_code)]
use std::cell::RefCell;

use ephact::application::{
    dtos::PullJobImageRequest, ports::outbound::pull_job_image_port::PullJobImagePort,
};

/// Returns a prepared image, recording the runner labels it was asked about.
pub struct FakePullJobImagePort {
    result: Result<String, String>,
    pub requested_labels: RefCell<Vec<Option<String>>>,
}

impl FakePullJobImagePort {
    pub fn returning(image: &str) -> Self {
        Self {
            result: Ok(image.to_string()),
            requested_labels: RefCell::new(Vec::new()),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            result: Err(message.to_string()),
            requested_labels: RefCell::new(Vec::new()),
        }
    }
}

impl PullJobImagePort for FakePullJobImagePort {
    fn execute(
        &self,
        request: PullJobImageRequest<'_>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.requested_labels
            .borrow_mut()
            .push(request.runs_on.map(str::to_string));
        self.result.clone().map_err(Into::into)
    }
}
