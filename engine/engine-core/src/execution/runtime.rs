//! Execution runtime.
//
//! Coordinates scheduling, execution, and state transitions.

use super::{
    context::ExecutionContext,
    error::ExecutionError,
    executor::Executor,
    scheduler::Scheduler,
    state::EngineState,
    task::Task,
};

/// Main execution runtime.
#[derive(Debug)]
pub struct ExecutionRuntime<S, E> {
    scheduler: S,
    executor: E,
    state: EngineState,
}

impl<S, E> ExecutionRuntime<S, E>
where
    S: Scheduler,
    E: Executor,
{
    /// Create a new execution runtime.
    pub fn new(
        scheduler: S,
        executor: E,
        state: EngineState,
    ) -> Self {
        Self {
            scheduler,
            executor,
            state,
        }
    }

    /// Execute the next scheduled task.
    pub fn step<T>(
        &mut self,
        context: &ExecutionContext,
    ) -> Result<(), ExecutionError>
    where
        T: Task,
        E: Executor<Task = T>,
    {
        let task = self.scheduler.next()
            .ok_or(ExecutionError::NoTask)?;

        let result = self.executor.execute(
            task,
            context,
        )?;

        self.state.apply(result)?;

        Ok(())
    }

    /// Access current engine state.
    pub fn state(&self) -> &EngineState {
        &self.state
    }
}