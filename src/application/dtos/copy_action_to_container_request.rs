use std::path::Path;

use crate::application::ports::outbound::ContainerPort;

/// Request DTO for the
/// [`CopyActionToContainerPort`](crate::application::ports::outbound::copy_action_to_container_port::CopyActionToContainerPort)
/// inbound port.
pub struct CopyActionToContainerRequest<'a> {
    /// Directory holding the action on the host.
    pub action_dir: &'a Path,
    /// Container the action is copied into.
    pub container: &'a dyn ContainerPort,
}
