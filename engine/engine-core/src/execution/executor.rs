//! Task execution engine.
//
//! Executes scheduled tasks within an execution context.

use super::{
    context::ExecutionContext,
    error::ExecutionError,
    task::Task,
};

/// Executor abstraction.
pub trait Executor {
    type Task: Task;
    type Output;

    /// Execute a task.
    fn execute(
        &self,
        task: Self::Task,
        context: &ExecutionContext,
    ) -> Result<Self::Output, ExecutionError>;
}


/// Default synchronous executor.
#[derive(Debug, Default)]
pub struct DefaultExecutor;

impl DefaultExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl<T> Executor for DefaultExecutor
where
    T: Task,
{
    type Task = T;
    type Output = T::Output;

    fn execute(
        &self,
        task: T,
        _context: &ExecutionContext,
    ) -> Result<Self::Output, ExecutionError> {
        task.execute()
            .map_err(|_| ExecutionError::TaskFailed)
    }
}