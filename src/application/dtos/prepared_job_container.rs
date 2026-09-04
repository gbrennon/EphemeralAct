use std::sync::Arc;

use crate::application::ports::outbound::container_port::ContainerPort;

/// Container a job's steps run in, with the name it was created under.
pub struct PreparedJobContainer {
    /// Handle to the running container.
    pub container: Arc<dyn ContainerPort>,
    /// Name the container was created with.
    pub container_name: String,
}
