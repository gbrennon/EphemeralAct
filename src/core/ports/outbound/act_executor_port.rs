use crate::core::shared_types::ExecutionResult;

pub trait ActExecutor {
    fn execute(&self, args: &[String]) -> Result<ExecutionResult, String>;
}
