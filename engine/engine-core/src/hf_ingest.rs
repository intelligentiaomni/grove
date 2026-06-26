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
    pub id: String,                     // Prefixed Document Hash (sha256_...)
    pub repo: String,                   // e.g., "HuggingFaceFW/fineweb-edu"
    pub split: String,
    pub parquet_file: String,
    pub byte_range: String,             // Active HTTP range scope
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
pub enum HfIngestError {
    NetworkStarvation,
    Network(reqwest::Error),
    Serialization(serde_json::Error),
    Io(std::io::Error),
}

impl Display for HfIngestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NetworkStarvation => write!(f, "Network buffer empty, pipeline starved"),
            Self::Network(err) => write!(f, "HTTP request failed: {err}"),
            Self::Serialization(err) => write!(f, "Ledger JSON mapping failed: {err}"),
            Self::Io(err) => write!(f, "Local log writing stalled: {err}"),
        }
    }
}

impl std::error::Error for HfIngestError {}

// =========================================================================
// Core Streaming Execution Engine
// =========================================================================

/// Unified stream execution loop. Processes chunk blocks over the wire and 
/// drives zero-copy topic filters, profiling layers, and transaction logs.
pub fn stream_range_request_loop(
    orchestrator: &mut PipelineOrchestrator,
    mut mock_network_stream: &[u8]
) -> Result<(), HfIngestError> {
    // Upgraded to a compact 4KB stack frame window for tighter CPU cache line alignment
    let mut buffer = [0u8; 4096];
    let source_hash = "sha256_9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    loop {
        // Simulate reading raw bytes from an authenticated HTTP connection
        let bytes_read = mock_network_stream.read(&mut buffer)
            .map_err(|e| HfIngestError::Io(e))?;

        if bytes_read > 0 {
            // Fixes borrow conflict: Read directly from orchestrator's own optimizer
            orchestrator.optimizer.set_starvation_state(false);
            
            // Map our buffer into the M6 profiled ingestion core
            orchestrator.ingest_stream_block_monitored(source_hash, &buffer[..bytes_read]);
        } else {
            // Socket buffer empty: flag starvation via orchestrator reference
            orchestrator.optimizer.set_starvation_state(true);

            // Construct exportable schema frame for the optimizer pass
            let mut exportable_ledger = UnifiedLedger {
                research_question: "How can early stopping criteria be optimized?".to_string(),
                sources: SourcesContainer {
                    datasets: orchestrator.state.datasets.clone(),
                    literature: orchestrator.state.literature.clone(),
                },
            };

            // Interleave maintenance using internal orchestrator components directly
            let performed_work = orchestrator.optimizer.interleave_idle_maintenance(
                &mut orchestrator.action_history,
                &mut exportable_ledger
            );

            // Apply optimized updates back to the orchestrator state
            orchestrator.state.datasets = exportable_ledger.sources.datasets;

            if !performed_work && mock_network_stream.is_empty() {
                // Terminate loop once the complete payload stream has been drained cleanly
                println!("[STREAM] Network pipeline transmission finished cleanly.");
                break;
            }
        }
    }

    Ok(())
}

/// Helper to generate safe cryptographic strings for provenance data keys
pub fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
