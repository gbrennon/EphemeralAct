use std::collections::HashMap;

use crate::{
    application::dtos::{ExecResult, FileEntry, RunnerContext},
    domain::errors::ContainerError,
};

/// Outbound port for working inside one running container.
///
/// The application layer coordinates a run in terms of "execute this command"
/// and "move these files"; which runtime backs the container is an
/// infrastructure concern behind this port.
pub trait ContainerPort: Send + Sync {
    /// Executes a command inside the container.
    fn exec(
        &self,
        cmd: &[String],
        workdir: Option<&str>,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError>;

    /// Copies the given entries into the container at `container_path`.
    fn copy_to(&self, container_path: &str, entries: &[FileEntry]) -> Result<(), ContainerError>;

    /// Reads the entries stored under `container_path` out of the container.
    fn copy_from(&self, container_path: &str) -> Result<Vec<FileEntry>, ContainerError>;

    /// Removes the container.
    fn remove(&self) -> Result<(), ContainerError>;

    /// Reports the runner context the container exposes to steps.
    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError>;
}
