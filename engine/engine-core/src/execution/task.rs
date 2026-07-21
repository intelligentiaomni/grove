//! Executable task abstractions.
//
//! Tasks are the atomic units processed by the execution engine.

/// A unit of executable work.
pub trait Task {
    type Output;
    type Error;

    /// Execute the task.
    fn execute(&self) -> Result<Self::Output, Self::Error>;
}