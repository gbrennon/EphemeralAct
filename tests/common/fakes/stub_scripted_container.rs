#![allow(dead_code)]
use std::collections::HashMap;

use ephact::application::ports::outbound::{
    ContainerError, ContainerPort, ExecResult, FileEntry, RunnerContext,
};

/// Container that answers every execution with one prepared result.
pub struct StubScriptedContainer {
    exit_code: i64,
    stdout: String,
}

impl StubScriptedContainer {
    pub fn answering(exit_code: i64, stdout: &str) -> Self {
        Self {
            exit_code,
            stdout: stdout.to_string(),
        }
    }
}

impl ContainerPort for StubScriptedContainer {
    fn exec(
        &self,
        _cmd: &[String],
        _workdir: Option<&str>,
        _env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        Ok(ExecResult {
            exit_code: self.exit_code,
            stdout: self.stdout.clone(),
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
