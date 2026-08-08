use std::collections::HashMap;

/// Configuration for creating a container.
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Docker image name (e.g. "catthehacker/ubuntu:act-latest")
    pub image: String,
    /// Target platform (e.g. "linux/amd64")
    pub platform: Option<String>,
    /// Environment variables injected into the container
    pub env: HashMap<String, String>,
    /// Volume binds in "host_path:container_path" format
    pub binds: Vec<String>,
    /// Working directory inside the container
    pub workdir: Option<String>,
    /// Command to run (overrides image CMD)
    pub cmd: Option<Vec<String>>,
    /// Entrypoint override
    pub entrypoint: Option<Vec<String>>,
    /// Network mode (e.g. "host", "bridge")
    pub network: Option<String>,
    /// Container name
    pub name: Option<String>,
}
