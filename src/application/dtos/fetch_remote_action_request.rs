use crate::domain::value_objects::RemoteActionReference;

/// Request DTO for the
/// [`FetchRemoteActionPort`](crate::application::ports::outbound::fetch_remote_action_port::FetchRemoteActionPort)
/// inbound port.
pub struct FetchRemoteActionRequest<'a> {
    /// Reference naming the action to retrieve.
    pub reference: &'a RemoteActionReference,
}
