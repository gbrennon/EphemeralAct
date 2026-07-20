
use crate::infrastructure::{execute_act_command, ExecutionResult};
use std::process;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_act_command_success() {
        let result = execute_act_command(vec!["run", "--test", "github-actions-test".to_string()]).await;
        
        assert!(result.unwrap().success);
        assert!(!result.unwrap().stderr.is_empty());
    }

    #[tokio::test]
    async fn test_act_command_failure() {
        let result = execute_act_command(vec!["run", "--test", "invalid-test".to_string()]).await;
        
        assert!(!result.unwrap().success);
        assert!(result.unwrap().stderr.contains("No such test"));
    }
}