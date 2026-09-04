#![allow(dead_code)]
use std::collections::HashMap;

use ephact::application::dtos::ExecResult;
use ephact::application::dtos::FileEntry;
use ephact::application::dtos::RunnerContext;
use ephact::application::ports::outbound::container_port::ContainerPort;
use ephact::domain::errors::ContainerError;

/// Container that fails every operation, for tests that need to see how a
/// service surfaces a container failure.
pub struct StubFailingContainer;

impl ContainerPort for StubFailingContainer {
    fn exec(
        &self,
        _cmd: &[String],
        _workdir: Option<&str>,
        _env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        Err(ContainerError::ExecutionFailed(
            "stub".into(),
            "exec refused".into(),
        ))
    }

    fn copy_to(&self, _path: &str, _entries: &[FileEntry]) -> Result<(), ContainerError> {
        Err(ContainerError::Internal("copy refused".into()))
    }

    fn copy_from(&self, _path: &str) -> Result<Vec<FileEntry>, ContainerError> {
        Err(ContainerError::Internal("copy refused".into()))
    }

    fn remove(&self) -> Result<(), ContainerError> {
        Err(ContainerError::RemovalFailed(
            "stub".into(),
            "removal refused".into(),
        ))
    }

    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
        Err(ContainerError::NotAvailable)
    }
}
