#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ephact::application::ports::outbound::{
    ContainerError, ContainerPort, ExecResult, FileEntry, RunnerContext,
};

/// Container that succeeds at everything and records what it was asked to run
/// and copy. Every clone observes the same recording.
#[derive(Clone, Default)]
pub struct StubRecordingContainer {
    executed_commands: Rc<RefCell<Vec<Vec<String>>>>,
    exec_environments: Rc<RefCell<Vec<HashMap<String, String>>>>,
    copied_paths: Rc<RefCell<Vec<String>>>,
    copied_files: Rc<RefCell<Vec<Vec<FileEntry>>>>,
}

impl StubRecordingContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn executed_commands(&self) -> Vec<Vec<String>> {
        self.executed_commands.borrow().clone()
    }

    pub fn exec_environments(&self) -> Vec<HashMap<String, String>> {
        self.exec_environments.borrow().clone()
    }

    pub fn copied_paths(&self) -> Vec<String> {
        self.copied_paths.borrow().clone()
    }

    pub fn copied_files(&self) -> Vec<Vec<FileEntry>> {
        self.copied_files.borrow().clone()
    }
}

impl ContainerPort for StubRecordingContainer {
    fn exec(
        &self,
        cmd: &[String],
        _workdir: Option<&str>,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        self.executed_commands.borrow_mut().push(cmd.to_vec());
        self.exec_environments.borrow_mut().push(env.clone());
        Ok(ExecResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        })
    }

    fn copy_to(&self, path: &str, entries: &[FileEntry]) -> Result<(), ContainerError> {
        self.copied_paths.borrow_mut().push(path.to_string());
        self.copied_files.borrow_mut().push(entries.to_vec());
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
