#[cfg(test)]
mod tests {
    use crate::scenarios::remote_action_pipeline_run::RemoteActionPipelineRun;

    struct RemoteActionPipelineTests;

    impl RemoteActionPipelineTests {
        fn a_workflow_using_a_remote_action_succeeds() {
            let run = RemoteActionPipelineRun::execute();
            assert_eq!(run.outcome, Ok(()));
        }

        fn the_action_is_fetched_from_the_forge_named_in_the_reference() {
            let run = RemoteActionPipelineRun::execute();
            let fetched = run.fetcher.fetched();
            let reference = fetched.first().unwrap();
            assert_eq!(reference.host(), "data.forgejo.org");
            assert_eq!(reference.owner(), "actions");
            assert_eq!(reference.repo(), "setup-node");
            assert_eq!(reference.git_ref(), "v4");
        }

        fn the_fetched_action_is_copied_into_the_container() {
            let run = RemoteActionPipelineRun::execute();
            assert!(
                run.activity
                    .copied_to_path_containing(RemoteActionPipelineRun::CONTAINER_ACTIONS_ROOT)
            );
        }

        fn the_javascript_entry_point_is_executed_in_the_container() {
            let run = RemoteActionPipelineRun::execute();
            assert!(
                run.activity
                    .ran_command_containing(RemoteActionPipelineRun::ENTRY_POINT_FILE)
            );
        }

        fn action_inputs_are_exposed_as_environment_variables() {
            let run = RemoteActionPipelineRun::execute();
            assert!(
                run.activity
                    .ran_command_with_environment(RemoteActionPipelineRun::INPUT_VARIABLE, "20")
            );
        }

        fn the_step_after_the_remote_action_still_runs() {
            let run = RemoteActionPipelineRun::execute();
            assert!(
                run.activity
                    .ran_script(RemoteActionPipelineRun::TOOLCHAIN_SCRIPT)
            );
        }
    }

    #[test]
    fn a_workflow_using_a_remote_action_succeeds() {
        RemoteActionPipelineTests::a_workflow_using_a_remote_action_succeeds();
    }

    #[test]
    fn the_action_is_fetched_from_the_forge_named_in_the_reference() {
        RemoteActionPipelineTests::the_action_is_fetched_from_the_forge_named_in_the_reference();
    }

    #[test]
    fn the_fetched_action_is_copied_into_the_container() {
        RemoteActionPipelineTests::the_fetched_action_is_copied_into_the_container();
    }

    #[test]
    fn the_javascript_entry_point_is_executed_in_the_container() {
        RemoteActionPipelineTests::the_javascript_entry_point_is_executed_in_the_container();
    }

    #[test]
    fn action_inputs_are_exposed_as_environment_variables() {
        RemoteActionPipelineTests::action_inputs_are_exposed_as_environment_variables();
    }

    #[test]
    fn the_step_after_the_remote_action_still_runs() {
        RemoteActionPipelineTests::the_step_after_the_remote_action_still_runs();
    }
}