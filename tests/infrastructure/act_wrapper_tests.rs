use ephemeral_act::infrastructure::{execute_act_command, ExecutionResult};

#[test]
#[ignore = "requires GitHub Actions workflow files in the repository"]
fn test_act_command_success() {
    let result = execute_act_command(vec!["run".to_string(), "--test".to_string(), "github-actions-test".to_string()]);

    let execution = result.unwrap();
    assert!(execution.success);
    assert!(!execution.stderr.is_empty());
}

#[test]
#[ignore = "requires GitHub Actions workflow files in the repository"]
fn test_act_command_failure() {
    let result = execute_act_command(vec!["run".to_string(), "--test".to_string(), "invalid-test".to_string()]);

    let execution = result.unwrap();
    assert!(!execution.success);
    assert!(execution.stderr.contains("No such test"));
}
