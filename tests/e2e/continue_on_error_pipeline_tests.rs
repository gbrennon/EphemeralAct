#[cfg(test)]
mod tests {
    use crate::scenarios::continue_on_error_pipeline_run::ContinueOnErrorPipelineRun;

    struct ContinueOnErrorPipelineTests;

    impl ContinueOnErrorPipelineTests {
        fn tolerated_step_failures_keep_the_run_successful() {
            let run = ContinueOnErrorPipelineRun::execute();
            assert_eq!(run.outcome, Ok(()));
        }

        fn a_tolerated_failure_does_not_stop_the_following_step() {
            let run = ContinueOnErrorPipelineRun::execute();
            assert!(run.activity.ran_before(
                ContinueOnErrorPipelineRun::DEPENDENCY_SCRIPT,
                ContinueOnErrorPipelineRun::LICENSE_SCRIPT
            ));
        }
    }

    #[test]
    fn tolerated_step_failures_keep_the_run_successful() {
        ContinueOnErrorPipelineTests::tolerated_step_failures_keep_the_run_successful();
    }

    #[test]
    fn a_tolerated_failure_does_not_stop_the_following_step() {
        ContinueOnErrorPipelineTests::a_tolerated_failure_does_not_stop_the_following_step();
    }
}
