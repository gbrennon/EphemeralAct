#[cfg(test)]
mod tests {
    use crate::scenarios::failing_pipeline_run::FailingPipelineRun;

    struct FailingPipelineTests;

    impl FailingPipelineTests {
        fn a_workflow_whose_steps_fail_reports_a_failed_run() {
            let run = FailingPipelineRun::execute();
            assert_eq!(run.outcome, Err("workflow failed".to_string()));
        }

        fn the_failing_shell_step_was_executed() {
            let run = FailingPipelineRun::execute();
            assert!(run.activity.ran_script(FailingPipelineRun::SUITE_SCRIPT));
        }

        fn the_failing_composite_action_step_was_executed() {
            let run = FailingPipelineRun::execute();
            assert!(run.activity.ran_script(FailingPipelineRun::RELEASE_SCRIPT));
        }

        fn containers_are_stopped_even_when_the_run_fails() {
            let run = FailingPipelineRun::execute();
            assert_eq!(run.activity.stopped_containers().len(), 2);
        }
    }

    #[test]
    fn a_workflow_whose_steps_fail_reports_a_failed_run() {
        FailingPipelineTests::a_workflow_whose_steps_fail_reports_a_failed_run();
    }

    #[test]
    fn the_failing_shell_step_was_executed() {
        FailingPipelineTests::the_failing_shell_step_was_executed();
    }

    #[test]
    fn the_failing_composite_action_step_was_executed() {
        FailingPipelineTests::the_failing_composite_action_step_was_executed();
    }

    #[test]
    fn containers_are_stopped_even_when_the_run_fails() {
        FailingPipelineTests::containers_are_stopped_even_when_the_run_fails();
    }
}