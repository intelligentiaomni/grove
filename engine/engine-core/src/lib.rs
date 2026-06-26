//! engine-core
//!
//! Shared core types, traits, and utilities used across the engine:
//! - math + geometry primitives
//! - engine config
//! - ECS-facing abstractions
//! - error types
//! - common data structures
//! - line-rate data ingestion & research lineage logs

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

// === High-Performance Data Streaming & Token Filtering Additions ===
pub mod hf_ingest;
pub mod token_filter;
pub mod pipeline;

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

// === Data Streaming Re-exports ===
pub use hf_ingest::{stream_range_request_loop, UnifiedLedger, DatasetSource, LiteratureSource};
pub use pipeline::PipelineOrchestrator;
pub use token_filter::SinglePassMatcher;

// ==========================================
// Research Ledger Structural Model
// ==========================================
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub status: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CorpusItem {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: u32,
    pub abstract_text: String, // Maps JSON "abstract" to Rust-safe variable name
    pub relevance_score: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResearchLedger {
    pub research_question: String,
    pub search_queries: Vec<SearchQuery>,
    #[serde(rename = "corpus")] // Direct mapping match
    pub corpus: Vec<CorpusItem>,
}

/// Parses the JSON ledger file and loads it into memory for core processing
pub fn load_ledger_from_disk(file_path: &str) -> Result<ResearchLedger, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let ledger: ResearchLedger = serde_json::from_str(&contents)?;
    Ok(ledger)
}

// ==========================================
// Legacy Insight Execution Passthrough
// ==========================================
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