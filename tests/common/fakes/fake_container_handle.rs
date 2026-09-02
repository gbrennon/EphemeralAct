#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ephemeral_act::core::ports::outbound::{
    ContainerError, ContainerPort, ExecResult, FileEntry, RunnerContext,
};

/// Container handle a [`super::fake_runtime::FakeRuntime`] creates.
///
/// Records what it was asked to execute and copy, and answers each execution
/// with the next queued result, or with a successful empty result once the
/// queue is drained. Reading a file the runner writes (`cat`) fails, the way a
/// container that never wrote the file behaves.
pub struct FakeContainerHandle {
    exec_results: Rc<RefCell<Vec<ExecResult>>>,
    executed_commands: Rc<RefCell<Vec<Vec<String>>>>,
    exec_environments: Rc<RefCell<Vec<HashMap<String, String>>>>,
    copied_paths: Rc<RefCell<Vec<String>>>,
}

impl FakeContainerHandle {
    pub fn new(
        exec_results: Rc<RefCell<Vec<ExecResult>>>,
        executed_commands: Rc<RefCell<Vec<Vec<String>>>>,
        exec_environments: Rc<RefCell<Vec<HashMap<String, String>>>>,
        copied_paths: Rc<RefCell<Vec<String>>>,
    ) -> Self {
        Self {
            exec_results,
            executed_commands,
            exec_environments,
            copied_paths,
        }
    }
}

impl ContainerPort for FakeContainerHandle {
    fn exec(
        &self,
        cmd: &[String],
        _workdir: Option<&str>,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        if cmd.first().map(String::as_str) == Some("cat") {
            return Err(ContainerError::ExecutionFailed(
                "fake".into(),
                "No such file or directory".into(),
            ));
        }

        self.executed_commands.borrow_mut().push(cmd.to_vec());
        self.exec_environments.borrow_mut().push(env.clone());

        let mut results = self.exec_results.borrow_mut();
        if results.is_empty() {
            Ok(ExecResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        } else {
            Ok(results.remove(0))
        }
    }

    fn copy_to(&self, path: &str, _entries: &[FileEntry]) -> Result<(), ContainerError> {
        self.copied_paths.borrow_mut().push(path.to_string());
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
