#![allow(dead_code)]
use std::collections::HashMap;

use ephemeral_act::core::ports::outbound::{
    ContainerError, ContainerPort, ExecResult, FileEntry, RunnerContext,
};

/// Container that succeeds at everything without recording anything, for tests
/// that need a container handle but never inspect it.
pub struct StubContainer;

impl ContainerPort for StubContainer {
    fn exec(
        &self,
        _cmd: &[String],
        _workdir: Option<&str>,
        _env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        Ok(ExecResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        })
    }

    fn copy_to(&self, _path: &str, _entries: &[FileEntry]) -> Result<(), ContainerError> {
        Ok(())
    }

    fn copy_from(&self, _path: &str) -> Result<Vec<FileEntry>, ContainerError> {
        Ok(vec![])
    }

    fn remove(&self) -> Result<(), ContainerError> {
        Ok(())
    }

    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
        Ok(RunnerContext::default())
    }
}
