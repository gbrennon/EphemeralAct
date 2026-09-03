use std::collections::HashMap;

use ephact::application::ports::outbound::{
    ContainerError, ContainerPort, ExecResult, FileEntry, RunnerContext,
};

use crate::support::container_activity::ContainerActivity;

/// Container of the scenario where every command the runner issues exits with
/// a failure status.
pub struct FailingContainer {
    activity: ContainerActivity,
}

impl FailingContainer {
    pub fn recording(activity: ContainerActivity) -> Self {
        Self { activity }
    }
}

impl ContainerPort for FailingContainer {
    fn exec(
        &self,
        cmd: &[String],
        _workdir: Option<&str>,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError> {
        self.activity.record_command(cmd, env);
        Ok(ExecResult {
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
        })
    }

    fn copy_to(&self, container_path: &str, _entries: &[FileEntry]) -> Result<(), ContainerError> {
        self.activity.record_copy(container_path);
        Ok(())
    }

    fn copy_from(&self, _container_path: &str) -> Result<Vec<FileEntry>, ContainerError> {
        Ok(Vec::new())
    }

    fn remove(&self) -> Result<(), ContainerError> {
        Ok(())
    }

    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
        Ok(RunnerContext::default())
    }
}
