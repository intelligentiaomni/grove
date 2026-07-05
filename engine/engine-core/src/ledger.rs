use crate::pipeline::PipelineOrchestrator;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Read;

// =========================================================================
// Shared Container Schemas for the Transactional Ledger
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatasetSource {
    pub id: String, // Prefixed Document Hash (sha256_...)
    pub repo: String,
    pub split: String,
    pub parquet_file: String,
    pub byte_range: String,
    pub extracted_topics: Vec<String>,
    pub provenance_node_bound: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LiteratureSource {
    pub id: String,
    pub repo: String,
    pub title: String,
    pub authors: Vec<String>,
    pub doi_or_url: String,
    pub extracted_nodes: Vec<String>,
    pub download_timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourcesContainer {
    pub datasets: Vec<DatasetSource>,
    pub literature: Vec<LiteratureSource>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnifiedLedger {
    pub research_question: String,
    pub sources: SourcesContainer,
}

// =========================================================================
// Error Handling Architecture
// =========================================================================

#[derive(Debug)]
pub enum LedgerStreamError {
    StreamStarvation,
    Serialization(serde_json::Error),
    Io(std::io::Error),
}

impl Display for LedgerStreamError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::StreamStarvation => write!(f, "validated input stream is starved"),
            Self::Serialization(err) => write!(f, "ledger JSON mapping failed: {err}"),
            Self::Io(err) => write!(f, "local ledger operation failed: {err}"),
        }
    }
}

impl std::error::Error for LedgerStreamError {}

// =========================================================================
// Core Validated Stream Execution Engine
// =========================================================================

/// Processes already validated byte blocks and drives topic filters, profiling
/// layers, and transaction logs without owning transport or deserialization.
pub fn process_verified_stream_loop(
    orchestrator: &mut PipelineOrchestrator,
    mut verified_stream: &[u8],
) -> Result<(), LedgerStreamError> {
    let mut buffer = [0u8; 4096];
    let source_hash = "sha256_9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    loop {
        let bytes_read = verified_stream
            .read(&mut buffer)
            .map_err(LedgerStreamError::Io)?;

        if bytes_read > 0 {
            orchestrator.optimizer.set_starvation_state(false);
            orchestrator.ingest_stream_block_monitored(source_hash, &buffer[..bytes_read]);
        } else {
            orchestrator.optimizer.set_starvation_state(true);

            let mut exportable_ledger = UnifiedLedger {
                research_question: "How can early stopping criteria be optimized?".to_string(),
                sources: SourcesContainer {
                    datasets: orchestrator.state.datasets.clone(),
                    literature: orchestrator.state.literature.clone(),
                },
            };

            let performed_work = orchestrator.optimizer.interleave_idle_maintenance(
                &mut orchestrator.action_history,
                &mut exportable_ledger,
            );

            orchestrator.state.datasets = exportable_ledger.sources.datasets;

            if !performed_work && verified_stream.is_empty() {
                println!("[STREAM] Verified pipeline transmission finished cleanly.");
                break;
            }
        }
    }

    Ok(())
}

/// Helper to generate safe cryptographic strings for provenance data keys.
pub fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
