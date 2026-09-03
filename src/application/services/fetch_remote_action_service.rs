use std::path::PathBuf;

use crate::{
    application::{
        dtos::FetchRemoteActionRequest,
        ports::{
            inbound::fetch_remote_action_port::FetchRemoteActionPort, outbound::ActionFetcherPort,
        },
    },
    domain::errors::ActionError,
};

/// Service that retrieves an action published on a forge, narrowing the result
/// to the subdirectory the reference names when it names one.
pub struct FetchRemoteActionService {
    fetcher: Box<dyn ActionFetcherPort>,
}

impl FetchRemoteActionService {
    pub fn new(fetcher: Box<dyn ActionFetcherPort>) -> Self {
        Self { fetcher }
    }
}

impl FetchRemoteActionPort for FetchRemoteActionService {
    fn execute(&self, request: FetchRemoteActionRequest<'_>) -> Result<PathBuf, ActionError> {
        let fetched = self.fetcher.fetch(request.reference)?;
        Ok(match request.reference.directory() {
            Some(directory) => fetched.join(directory),
            None => fetched,
        })
    }
}
