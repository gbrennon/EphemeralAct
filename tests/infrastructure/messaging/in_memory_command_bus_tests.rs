use std::{collections::HashMap, path::PathBuf, sync::Arc};

use ephact::application::commands::{ExecuteActionCommand, ExecuteWorkflowCommand};
use ephact::{
    application::{
        dtos::{
            ExecuteActionRequest, ExecuteActionResponse, ExecutedStep, JobExecution, JobSummary,
            WorkflowExecution,
        },
        ports::{
            inbound::{
                execute_action_port::ExecuteActionPort, execute_job_port::ExecuteJobPort,
                execute_step_port::ExecuteStepPort, execute_workflow_port::ExecuteWorkflowPort,
            },
            outbound::CommandBusPort,
        },
    },
    domain::{
        ActRunConfig, RepoPath, Repository, RepositoryName, errors::StepError,
        expression::EvalContext, workflow::Step,
    },
    infrastructure::{
        actions::ActionCommandHandler, jobs::JobCommandHandler, messaging::InMemoryCommandBus,
        steps::StepCommandHandler, workflows::WorkflowCommandHandler,
    },
};

use crate::common::fakes::stub_container::StubContainer;

struct StubWorkflowPort;
impl ExecuteWorkflowPort for StubWorkflowPort {
    fn execute(
        &self,
        _request: ephact::application::dtos::ExecuteWorkflowRequest<'_>,
    ) -> Result<WorkflowExecution, Box<dyn std::error::Error>> {
        Ok(WorkflowExecution {
            workflow_name: "dispatched-wf".into(),
            job_summaries: Vec::new(),
            container_names: vec!["c1".into()],
            success: true,
        })
    }
}

struct StubJobPort;
impl ExecuteJobPort for StubJobPort {
    fn execute(
        &self,
        _request: ephact::application::dtos::ExecuteJobRequest<'_>,
    ) -> Result<JobExecution, Box<dyn std::error::Error>> {
        Ok(JobExecution {
            job_summary: JobSummary {
                job_id: "j1".into(),
                name: Some("job 1".into()),
                steps: Vec::new(),
                success: true,
            },
            container_name: "c1".into(),
        })
    }
}

struct StubStepPort;
impl ExecuteStepPort for StubStepPort {
    fn execute(
        &self,
        request: ephact::application::dtos::ExecuteStepRequest<'_>,
    ) -> Result<ExecutedStep, StepError> {
        Ok(ExecutedStep {
            step: request.step.clone(),
            response: ExecuteActionResponse {
                exit_code: 0,
                stdout: "step out".into(),
                stderr: String::new(),
            },
        })
    }
}

struct StubActionPort;
impl ExecuteActionPort for StubActionPort {
    fn execute(&self, _request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError> {
        Ok(ExecuteActionResponse {
            exit_code: 0,
            stdout: "action out".into(),
            stderr: String::new(),
        })
    }
}

#[test]
fn command_bus_dispatches_workflow_to_workflow_handler() {
    let bus = InMemoryCommandBus::new(
        Box::new(WorkflowCommandHandler::new(Box::new(StubWorkflowPort))),
        Box::new(JobCommandHandler::new(Box::new(StubJobPort))),
        Box::new(StepCommandHandler::new(Box::new(StubStepPort))),
        Box::new(ActionCommandHandler::new(Box::new(StubActionPort))),
    );

    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    let repository = Repository::new(
        RepoPath::new(tmp.path().to_path_buf()).unwrap(),
        RepositoryName::new("test-repo".into()).unwrap(),
    );

    let cmd = ExecuteWorkflowCommand::new(
        "name: CI\non: [push]\n".to_string(),
        ActRunConfig::new(),
        repository,
    );

    let result = bus.dispatch_workflow(cmd).unwrap();
    assert_eq!(result.workflow_name, "dispatched-wf");
}

#[test]
fn command_bus_dispatches_action_to_action_handler() {
    let bus = InMemoryCommandBus::new(
        Box::new(WorkflowCommandHandler::new(Box::new(StubWorkflowPort))),
        Box::new(JobCommandHandler::new(Box::new(StubJobPort))),
        Box::new(StepCommandHandler::new(Box::new(StubStepPort))),
        Box::new(ActionCommandHandler::new(Box::new(StubActionPort))),
    );

    let step: Step = serde_yaml::from_str("uses: actions/checkout@v4").unwrap();
    let cmd = ExecuteActionCommand::new(
        "actions/checkout@v4".into(),
        step,
        PathBuf::from("/repo"),
        HashMap::new(),
        EvalContext::new(),
        Arc::new(StubContainer),
    );

    let result = bus.dispatch_action(cmd).unwrap();
    assert_eq!(result.stdout, "action out");
    assert_eq!(result.exit_code, 0);
}
