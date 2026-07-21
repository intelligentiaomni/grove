//! Execution state management.
//
//! Represents the current state of the execution engine.

use super::transition::StateTransition;

/// Current engine state.
#[derive(Debug, Clone)]
pub struct EngineState {
    /// Monotonically increasing state version.
    pub version: u64,

    /// Current state identifier.
    pub state_id: String,

    /// Last applied transition.
    pub last_transition: Option<StateTransition>,
}

impl EngineState {
    /// Create an initial state.
    pub fn new(state_id: impl Into<String>) -> Self {
        Self {
            version: 0,
            state_id: state_id.into(),
            last_transition: None,
        }
    }

    /// Apply a state transition.
    pub fn apply(
        &mut self,
        transition: StateTransition,
    ) -> Result<(), StateError> {
        self.version += 1;
        self.state_id = transition.to.clone();
        self.last_transition = Some(transition);

        Ok(())
    }
}