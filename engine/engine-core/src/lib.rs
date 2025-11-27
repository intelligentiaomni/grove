//! engine-core
//!
//! Shared core types, traits, and utilities used across the engine:
//! - math + geometry primitives
//! - engine config
//! - ECS-facing abstractions
//! - error types
//! - common data structures

pub mod math;
pub mod config;
pub mod error;
pub mod types;
pub mod ecs;
pub mod compute;

// Re-exports for convenience
pub use config::EngineConfig;
pub use error::EngineError;
pub use math::*;
pub use types::*;
pub use ecs::*;
pub use compute::wave::Wavefield;