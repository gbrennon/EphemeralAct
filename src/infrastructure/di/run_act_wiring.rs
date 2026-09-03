use std::sync::Arc;

use crate::application::{
    ports::outbound::{ContainerRuntimePort, EventPublisherPort, ImageMapperPort},
    services::{
        build_run_context_service::BuildRunContextService,
        build_step_context_service::BuildStepContextService,
        create_job_container_service::CreateJobContainerService,
        detect_workflow_file_service::DetectWorkflowFileService,
        execute_job_service::ExecuteJobService, execute_step_service::ExecuteStepService,
        execute_workflow_service::ExecuteWorkflowService,
        list_all_workflow_files_service::ListAllWorkflowFilesService,
        list_workflow_directory_service::ListWorkflowDirectoryService,
        load_workflow_service::LoadWorkflowService,
        merge_run_executions_service::MergeRunExecutionsService,
        prefix_step_path_service::PrefixStepPathService,
        prepare_job_container_service::PrepareJobContainerService,
        pull_job_image_service::PullJobImageService,
        read_step_env_exports_service::ReadStepEnvExportsService,
        read_step_exports_service::ReadStepExportsService,
        read_step_path_exports_service::ReadStepPathExportsService,
        request_action_execution_service::RequestActionExecutionService,
        resolve_named_workflow_file_service::ResolveNamedWorkflowFileService,
        resolve_workflow_files_service::ResolveWorkflowFilesService,
        run_act_service::RunActService, run_shell_step_service::RunShellStepService,
        summarize_step_service::SummarizeStepService,
    },
};

/// Assembles the service graph behind [`RunActService`].
///
/// Every construction site - production and test - goes through this builder,
/// so the graph is described in exactly one place.
pub struct RunActWiring;

impl RunActWiring {
    pub fn build(
        runtime: Arc<dyn ContainerRuntimePort>,
        image_mapper: Box<dyn ImageMapperPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> RunActService {
        let step_executor = Box::new(ExecuteStepService::new(
            Box::new(RequestActionExecutionService::new(event_publisher.clone())),
            Box::new(RunShellStepService::new()),
        ));
        let job_executor = Box::new(ExecuteJobService::new(
            Box::new(crate::infrastructure::runners::GitHubJobEnvironmentAdapter::new()),
            Box::new(PrepareJobContainerService::new(
                Box::new(PullJobImageService::new(runtime.clone(), image_mapper)),
                Box::new(CreateJobContainerService::new(runtime)),
            )),
            Box::new(PrefixStepPathService::new()),
            Box::new(BuildStepContextService::new()),
            step_executor,
            Box::new(SummarizeStepService::new()),
            Box::new(ReadStepExportsService::new(
                Box::new(ReadStepPathExportsService::new()),
                Box::new(ReadStepEnvExportsService::new()),
            )),
        ));
        RunActService::new(
            Box::new(ResolveWorkflowFilesService::new(
                Box::new(ListAllWorkflowFilesService::new(Box::new(
                    ListWorkflowDirectoryService::new(),
                ))),
                Box::new(ResolveNamedWorkflowFileService::new()),
                Box::new(DetectWorkflowFileService::new(Box::new(
                    ListWorkflowDirectoryService::new(),
                ))),
            )),
            Box::new(BuildRunContextService::new()),
            Box::new(ExecuteWorkflowService::new(
                Box::new(LoadWorkflowService::new()),
                job_executor,
            )),
            Box::new(MergeRunExecutionsService::new()),
            event_publisher,
        )
    }
}
