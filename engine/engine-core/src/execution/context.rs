//! Execution context.
//
//! Provides shared resources and metadata available during task execution.

use super::state::EngineState;

/// Runtime context provided to executable tasks.
#[derive(Debug)]
pub struct ExecutionContext {
    state: EngineState,
    run_id: String,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new(
        state: EngineState,
        run_id: impl Into<String>,
    ) -> Self {
        Self {
            state,
            run_id: run_id.into(),
        }
    }

    /// Access current engine state.
    pub fn state(&self) -> &EngineState {
        &self.state
    }

    /// Current execution identifier.
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
}