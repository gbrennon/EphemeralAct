use std::{cell::RefCell, collections::HashMap};

use ephemeral_act::core::ports::outbound::{
    Container, ContainerError, ExecResult, FileEntry, RunnerContext,
};

pub(super) struct FakeContainer {
    pub(super) exec_results: RefCell<Vec<ExecResult>>,
}

impl Container for FakeContainer {
    fn exec(
        &self,
        cmd: &[String],
        _workdir: Option<&str>,
        _env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        if let Some(preloaded) = self.exec_results.borrow_mut().pop() {
            return Ok(preloaded);
        }
        let output = std::process::Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .unwrap_or_else(|_| std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        Ok(ExecResult {
            exit_code: output.status.code().unwrap_or(0).into(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    fn copy_to(&self, _container_path: &str, _entries: &[FileEntry]) -> Result<(), ContainerError> {
        Ok(())
    }

    fn copy_from(&self, _container_path: &str) -> Result<Vec<FileEntry>, ContainerError> {
        Ok(vec![])
    }

    fn remove(&self) -> Result<(), ContainerError> {
        Ok(())
    }

    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
        unimplemented!()
    }
}
