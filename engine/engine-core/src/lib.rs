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
pub mod insight_accelerator;
pub mod math;
pub mod research;
pub mod scientific_agent;
pub mod telemetry;
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
    GraphTransitionRecord, LineageError,
};
pub use insight_accelerator::{
    CollisionResult, InsightAccelerator, ResearchVector, StructuralIntersection,
};
pub use math::*;
pub use research::{ResearchEvent, ResearchThread, SchedulerAction};
pub use scientific_agent::{
    AutomatedExperimentJob, ConstraintOperator, ScientificHypothesisNode, SymbolicConstraint,
};
pub use telemetry::{
    residual_error, simulate_twin, validate_telemetry, TelemetryBounds, TelemetryDecision,
    TelemetryPayload, TelemetryValidationReport, TwinKind, TwinState,
};
pub use types::*;
pub use visualization::{D3Link, D3Node, VisualizationManifest};

pub fn execute_insight_passthrough(
    research_text: &str,
    telemetry: &TelemetryPayload,
    previous: Option<&TelemetryPayload>,
    bounds: &[TelemetryBounds],
    twin_kind: TwinKind,
    baseline: TwinState,
) -> (CollisionResult, TelemetryValidationReport, TwinState) {
    let accelerator = InsightAccelerator::new();
    let collision = accelerator.calculate_conceptual_collision(research_text);
    let validation = validate_telemetry(telemetry, previous, bounds);
    let intervention = if matches!(validation.decision, TelemetryDecision::Clean) {
        telemetry.value
    } else {
        0.0
    };
    let twin_state = simulate_twin(twin_kind, baseline, intervention, 1);

    (collision, validation, twin_state)
}
