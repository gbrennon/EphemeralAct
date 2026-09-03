#![allow(clippy::arc_with_non_send_sync)]

mod fakes;
mod scenarios;
mod support;

mod continue_on_error_pipeline_tests;
mod delivery_pipeline_tests;
mod every_workflow_tests;
mod failing_pipeline_tests;
mod remote_action_pipeline_tests;
