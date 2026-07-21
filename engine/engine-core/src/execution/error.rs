//! Execution engine errors.
//
//! Centralized error types for task execution,
//! scheduling, state transitions, and runtime failures.

use std::fmt;

#[derive(Debug)]
pub enum ExecutionError {
    /// No executable task is available.
    NoTask,

    /// A task failed during execution.
    TaskFailed(String),

    /// The scheduler encountered an error.
    SchedulingFailed(String),

    /// State transition could not be applied.
    InvalidStateTransition(String),

    /// Runtime initialization or lifecycle failure.
    RuntimeFailure(String),

    /// External dependency failure.
    External(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::NoTask =>
                write!(f, "no task available"),

            Self::TaskFailed(msg) =>
                write!(f, "task failed: {msg}"),

            Self::SchedulingFailed(msg) =>
                write!(f, "scheduling failed: {msg}"),

            Self::InvalidStateTransition(msg) =>
                write!(f, "invalid state transition: {msg}"),

            Self::RuntimeFailure(msg) =>
                write!(f, "runtime failure: {msg}"),

            Self::External(msg) =>
                write!(f, "external error: {msg}"),
        }
    }
}

impl std::error::Error for ExecutionError {}