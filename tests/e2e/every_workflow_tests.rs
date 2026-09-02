#[cfg(test)]
mod tests {
    use crate::scenarios::every_workflow_run::EveryWorkflowRun;

    struct EveryWorkflowTests;

    impl EveryWorkflowTests {
        fn running_every_workflow_succeeds() {
            let run = EveryWorkflowRun::execute();
            assert_eq!(run.outcome, Ok(()));
        }

        fn the_jobs_of_every_workflow_file_are_executed() {
            let run = EveryWorkflowRun::execute();
            assert!(run.activity.ran_script(EveryWorkflowRun::LINT_SCRIPT));
            assert!(run.activity.ran_script(EveryWorkflowRun::UNIT_SCRIPT));
            assert!(
                run.activity
                    .ran_script(EveryWorkflowRun::INTEGRATION_SCRIPT)
            );
        }

        fn dependency_order_holds_inside_each_workflow_file() {
            let run = EveryWorkflowRun::execute();
            assert!(run.activity.ran_before(
                EveryWorkflowRun::UNIT_SCRIPT,
                EveryWorkflowRun::INTEGRATION_SCRIPT
            ));
        }

        fn one_container_is_created_and_stopped_per_job() {
            let run = EveryWorkflowRun::execute();
            assert_eq!(run.activity.pulled_images().len(), 3);
            assert_eq!(run.activity.stopped_containers().len(), 3);
        }
    }

    #[test]
    fn running_every_workflow_succeeds() {
        EveryWorkflowTests::running_every_workflow_succeeds();
    }

    #[test]
    fn the_jobs_of_every_workflow_file_are_executed() {
        EveryWorkflowTests::the_jobs_of_every_workflow_file_are_executed();
    }

    #[test]
    fn dependency_order_holds_inside_each_workflow_file() {
        EveryWorkflowTests::dependency_order_holds_inside_each_workflow_file();
    }

    #[test]
    fn one_container_is_created_and_stopped_per_job() {
        EveryWorkflowTests::one_container_is_created_and_stopped_per_job();
    }
}