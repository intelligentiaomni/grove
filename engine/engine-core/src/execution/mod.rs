//! Grove execution engine.
//
//! Responsible for deterministic execution of tasks,
//! state transitions, and runtime coordination.

pub mod context;
pub mod error;
pub mod executor;
pub mod runtime;
pub mod scheduler;
pub mod state;
pub mod task;
pub mod transition;

// Public execution API.
pub use context::*;
pub use error::*;
pub use executor::*;
pub use runtime::*;
pub use scheduler::*;
pub use state::*;
pub use task::*;
pub use transition::*;