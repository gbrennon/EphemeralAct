#[cfg(test)]
mod tests {
    use crate::{
        fakes::fixed_image_mapper::RUNNER_IMAGE,
        scenarios::delivery_pipeline_run::DeliveryPipelineRun,
    };

    struct DeliveryPipelineTests;

    impl DeliveryPipelineTests {
        fn a_pipeline_of_dependent_jobs_succeeds() {
            let run = DeliveryPipelineRun::execute();
            assert_eq!(run.outcome, Ok(()));
        }

        fn every_job_runs_in_its_own_container() {
            let run = DeliveryPipelineRun::execute();
            assert_eq!(run.activity.pulled_images(), vec![RUNNER_IMAGE; 3]);
        }

        fn jobs_run_in_the_order_their_dependencies_require() {
            let run = DeliveryPipelineRun::execute();
            assert!(run.activity.ran_before(
                DeliveryPipelineRun::CONTEXT_SCRIPT,
                DeliveryPipelineRun::PACKAGE_SCRIPT
            ));
            assert!(run.activity.ran_before(
                DeliveryPipelineRun::PACKAGE_SCRIPT,
                DeliveryPipelineRun::PUBLISH_SCRIPT
            ));
        }

        fn workflow_and_job_environments_are_resolved_in_scripts() {
            let run = DeliveryPipelineRun::execute();
            assert!(
                run.activity
                    .ran_script(DeliveryPipelineRun::ENVIRONMENT_SCRIPT)
            );
        }

        fn the_github_and_runner_contexts_are_resolved_in_scripts() {
            let run = DeliveryPipelineRun::execute();
            assert!(run.activity.ran_script(DeliveryPipelineRun::CONTEXT_SCRIPT));
        }

        fn command_line_inputs_and_secrets_are_resolved_in_scripts() {
            let run = DeliveryPipelineRun::execute();
            assert!(run.activity.ran_script(DeliveryPipelineRun::PUBLISH_SCRIPT));
        }

        fn composite_action_inputs_fall_back_to_their_declared_defaults() {
            let run = DeliveryPipelineRun::execute();
            assert!(run.activity.ran_script(DeliveryPipelineRun::PACKAGE_SCRIPT));
        }

        fn an_action_nested_in_a_composite_action_receives_inputs_and_secrets() {
            let run = DeliveryPipelineRun::execute();
            assert!(
                run.activity
                    .ran_script(DeliveryPipelineRun::CHECKSUM_SCRIPT)
            );
        }

        fn checking_out_the_repository_fetches_nothing() {
            let run = DeliveryPipelineRun::execute();
            assert_eq!(run.fetcher.fetched().len(), 0);
        }

        fn every_container_is_stopped_once_the_run_completes() {
            let run = DeliveryPipelineRun::execute();
            assert_eq!(run.activity.stopped_containers().len(), 3);
        }
    }

    #[test]
    fn a_pipeline_of_dependent_jobs_succeeds() {
        DeliveryPipelineTests::a_pipeline_of_dependent_jobs_succeeds();
    }

    #[test]
    fn every_job_runs_in_its_own_container() {
        DeliveryPipelineTests::every_job_runs_in_its_own_container();
    }

    #[test]
    fn jobs_run_in_the_order_their_dependencies_require() {
        DeliveryPipelineTests::jobs_run_in_the_order_their_dependencies_require();
    }

    #[test]
    fn workflow_and_job_environments_are_resolved_in_scripts() {
        DeliveryPipelineTests::workflow_and_job_environments_are_resolved_in_scripts();
    }

    #[test]
    fn the_github_and_runner_contexts_are_resolved_in_scripts() {
        DeliveryPipelineTests::the_github_and_runner_contexts_are_resolved_in_scripts();
    }

    #[test]
    fn command_line_inputs_and_secrets_are_resolved_in_scripts() {
        DeliveryPipelineTests::command_line_inputs_and_secrets_are_resolved_in_scripts();
    }

    #[test]
    fn composite_action_inputs_fall_back_to_their_declared_defaults() {
        DeliveryPipelineTests::composite_action_inputs_fall_back_to_their_declared_defaults();
    }

    #[test]
    fn an_action_nested_in_a_composite_action_receives_inputs_and_secrets() {
        DeliveryPipelineTests::an_action_nested_in_a_composite_action_receives_inputs_and_secrets();
    }

    #[test]
    fn checking_out_the_repository_fetches_nothing() {
        DeliveryPipelineTests::checking_out_the_repository_fetches_nothing();
    }

    #[test]
    fn every_container_is_stopped_once_the_run_completes() {
        DeliveryPipelineTests::every_container_is_stopped_once_the_run_completes();
    }
}
