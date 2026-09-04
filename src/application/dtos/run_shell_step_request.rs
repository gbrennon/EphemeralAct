use std::collections::HashMap;

use crate::application::ports::outbound::container_port::ContainerPort;
use crate::domain::workflow::Step;

pub struct RunShellStepRequest<'a> {
    pub step: &'a Step,

    pub container: &'a dyn ContainerPort,

    pub env: &'a HashMap<String, String>,
}
