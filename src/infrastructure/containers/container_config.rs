use crate::application::dtos::RunnerContext;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContainerConfig {
    pub image: String,

    pub platform: Option<String>,

    pub env: HashMap<String, String>,

    pub binds: Vec<String>,

    pub workdir: Option<String>,

    pub cmd: Option<Vec<String>>,

    pub entrypoint: Option<Vec<String>>,

    pub network: Option<String>,

    pub name: Option<String>,

    pub runner_context: RunnerContext,
}
