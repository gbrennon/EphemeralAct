#![allow(dead_code)]
use std::{error::Error, sync::Arc};

use ephact::{
    application::{dtos::WorkflowListItem, ports::outbound::WorkflowSourcePort},
    domain::Repository,
};
use parking_lot::Mutex;

#[derive(Default)]
struct FakeWorkflowSourceState {
    workflows: Vec<WorkflowListItem>,
    actions: Vec<String>,
    workflow_content: String,
    all_workflow_contents: Vec<String>,
    read_workflow_error: Option<String>,
    read_all_workflows_error: Option<String>,
    list_actions_error: Option<String>,
    list_workflows_error: Option<String>,
    read_workflow_calls: Vec<(Repository, Option<String>)>,
    read_all_workflows_calls: Vec<Repository>,
    list_actions_calls: Vec<Repository>,
    list_workflows_calls: Vec<Repository>,
}

/// Canned [`WorkflowSourcePort`] keeping its state behind `Arc<Mutex<_>>` so the
/// port stays `Send + Sync`, and so a clone held by the test can still observe
/// the calls made through the boxed clone owned by the service.
#[derive(Clone, Default)]
pub struct FakeWorkflowSource {
    state: Arc<Mutex<FakeWorkflowSourceState>>,
}

impl FakeWorkflowSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_workflows(self, workflows: Vec<WorkflowListItem>) -> Self {
        self.state.lock().workflows = workflows;
        self
    }

    pub fn with_actions(self, actions: Vec<String>) -> Self {
        self.state.lock().actions = actions;
        self
    }

    pub fn with_workflow_content(self, content: &str) -> Self {
        self.state.lock().workflow_content = content.to_string();
        self
    }

    pub fn with_all_workflow_contents(self, contents: Vec<String>) -> Self {
        self.state.lock().all_workflow_contents = contents;
        self
    }

    pub fn failing_read_workflow(self, message: &str) -> Self {
        self.state.lock().read_workflow_error = Some(message.to_string());
        self
    }

    pub fn failing_read_all_workflows(self, message: &str) -> Self {
        self.state.lock().read_all_workflows_error = Some(message.to_string());
        self
    }

    pub fn failing_list_actions(self, message: &str) -> Self {
        self.state.lock().list_actions_error = Some(message.to_string());
        self
    }

    pub fn failing_list_workflows(self, message: &str) -> Self {
        self.state.lock().list_workflows_error = Some(message.to_string());
        self
    }

    pub fn read_workflow_calls(&self) -> Vec<(Repository, Option<String>)> {
        self.state.lock().read_workflow_calls.clone()
    }

    pub fn read_all_workflows_calls(&self) -> Vec<Repository> {
        self.state.lock().read_all_workflows_calls.clone()
    }

    pub fn list_actions_calls(&self) -> Vec<Repository> {
        self.state.lock().list_actions_calls.clone()
    }

    pub fn list_workflows_calls(&self) -> Vec<Repository> {
        self.state.lock().list_workflows_calls.clone()
    }
}

impl WorkflowSourcePort for FakeWorkflowSource {
    fn read_workflow(
        &self,
        repository: &Repository,
        workflow_name: Option<&str>,
    ) -> Result<String, Box<dyn Error>> {
        let mut state = self.state.lock();
        state
            .read_workflow_calls
            .push((repository.clone(), workflow_name.map(str::to_string)));

        if let Some(message) = state.read_workflow_error.clone() {
            return Err(message.into());
        }

        Ok(state.workflow_content.clone())
    }

    fn read_all_workflows(&self, repository: &Repository) -> Result<Vec<String>, Box<dyn Error>> {
        let mut state = self.state.lock();
        state.read_all_workflows_calls.push(repository.clone());

        if let Some(message) = state.read_all_workflows_error.clone() {
            return Err(message.into());
        }

        Ok(state.all_workflow_contents.clone())
    }

    fn list_actions(&self, repository: &Repository) -> Result<Vec<String>, Box<dyn Error>> {
        let mut state = self.state.lock();
        state.list_actions_calls.push(repository.clone());

        if let Some(message) = state.list_actions_error.clone() {
            return Err(message.into());
        }

        Ok(state.actions.clone())
    }

    fn list_workflows(
        &self,
        repository: &Repository,
    ) -> Result<Vec<WorkflowListItem>, Box<dyn Error>> {
        let mut state = self.state.lock();
        state.list_workflows_calls.push(repository.clone());

        if let Some(message) = state.list_workflows_error.clone() {
            return Err(message.into());
        }

        Ok(state.workflows.clone())
    }
}
