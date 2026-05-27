//! engine-core
//!
//! Shared core types, traits, and utilities used across the engine:
//! - math + geometry primitives
//! - engine config
//! - ECS-facing abstractions
//! - error types
//! - common data structures

pub mod compute;
pub mod config;
pub mod correspondence;
pub mod ecs;
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod graph_lineage;
pub mod math;
pub mod research;
pub mod scientific_agent;
pub mod types;
pub mod visualization;

// Re-exports for convenience
pub use compute::wave::Wavefield;
pub use config::EngineConfig;
pub use correspondence::{CorrespondenceGraph, TopicNode};
pub use ecs::*;
pub use error::EngineError;
#[cfg(not(target_arch = "wasm32"))]
pub use graph_lineage::{
    GraphEdgeRecord, GraphLineageController, GraphLineageError, GraphNodeRecord,
};
pub use math::*;
pub use research::{ResearchEvent, ResearchThread, SchedulerAction};
pub use scientific_agent::{
    AutomatedExperimentJob, ConstraintOperator, ScientificHypothesisNode, SymbolicConstraint,
};
pub use types::*;
pub use visualization::{D3Link, D3Node, VisualizationManifest};
