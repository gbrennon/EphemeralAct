#![allow(dead_code)]
use std::collections::HashMap;

use ephact::application::dtos::ExecResult;
use ephact::application::dtos::FileEntry;
use ephact::application::dtos::RunnerContext;
use ephact::application::ports::outbound::container_port::ContainerPort;
use ephact::domain::errors::ContainerError;

/// Container that answers `cat <file>` with prepared contents, standing in for
/// a container whose steps wrote the runner's export files.
///
/// A file it was given no contents for fails the way a container that never
/// wrote it does.
pub struct StubExportingContainer {
    files: Vec<(String, String)>,
}

impl StubExportingContainer {
    pub fn holding(files: Vec<(String, String)>) -> Self {
        Self { files }
    }

    pub fn empty() -> Self {
        Self { files: Vec::new() }
    }
}

impl ContainerPort for StubExportingContainer {
    fn exec(
        &self,
        cmd: &[String],
        _workdir: Option<&str>,
        _env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        if cmd.first().map(String::as_str) != Some("cat") {
            return Ok(ExecResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            });
        }

        let path = cmd.get(1).cloned().unwrap_or_default();
        match self.files.iter().find(|(name, _)| name == &path) {
            Some((_, contents)) => Ok(ExecResult {
                exit_code: 0,
                stdout: contents.clone(),
                stderr: String::new(),
            }),
            None => Err(ContainerError::ExecutionFailed(
                "stub".into(),
                "No such file or directory".into(),
            )),
        }
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
