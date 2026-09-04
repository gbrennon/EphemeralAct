/// Information about the host container runtime.
#[derive(Debug, Clone)]
pub struct HostInfo {
    /// Operating system (e.g. "linux")
    pub os: String,
    /// Architecture (e.g. "amd64")
    pub arch: String,
    /// Container engine version string
    pub engine_version: String,
}
