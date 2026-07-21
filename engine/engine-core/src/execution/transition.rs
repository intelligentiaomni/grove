//! State transition model.
//
//! Represents deterministic changes produced by execution.

/// A single state transition.
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Previous state identifier.
    pub from: String,

    /// Resulting state identifier.
    pub to: String,

    /// Operation that caused the transition.
    pub operation: String,

    /// Execution timestamp.
    pub timestamp_utc: String,
}

impl StateTransition {
    /// Create a new state transition.
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        operation: impl Into<String>,
        timestamp_utc: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            operation: operation.into(),
            timestamp_utc: timestamp_utc.into(),
        }
    }
}