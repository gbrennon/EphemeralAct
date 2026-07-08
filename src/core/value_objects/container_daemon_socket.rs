use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerDaemonSocket(String);

impl ContainerDaemonSocket {
    pub fn new(socket: String) -> Self {
        Self(socket)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContainerDaemonSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_socket_string() {
        let socket = ContainerDaemonSocket::new("unix:///run/user/1000/podman/podman.sock".into());
        assert_eq!(socket.as_str(), "unix:///run/user/1000/podman/podman.sock");
    }

    #[test]
    fn display_returns_socket_string() {
        let socket = ContainerDaemonSocket::new("unix:///var/run/docker.sock".into());
        assert_eq!(socket.to_string(), "unix:///var/run/docker.sock");
    }
}
