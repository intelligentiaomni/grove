pub mod m1_state;
pub mod m2_storage;
pub mod m3_recording;
pub mod m5_replay;
pub mod m6_metrics;
pub mod m7_optimizer;

use crate::token_filter::SinglePassMatcher;
use crate::hf_ingest::{UnifiedLedger, SourcesContainer};
use crate::pipeline::m2_storage::LedgerStorageManager; 
use crate::pipeline::m6_metrics::{StreamProfiler, TransitionMetrics};
use m1_state::{PipelineState, IngestionAction, StateTransitionEngine};
use m3_recording::{PipelineObserver, AuditLogger};

pub struct PipelineOrchestrator {
    pub state: PipelineState,
    pub matcher: SinglePassMatcher,
    pub logger: AuditLogger,
    /// Memory collection tracking every event for verification or replay exports
    pub action_history: Vec<IngestionAction>, 
    /// Accumulated runtime telemetry records for pipeline benchmarking
    pub performance_log: Vec<TransitionMetrics>,
    /// Background optimizer task handler for thread runtime stalling
    pub optimizer: crate::pipeline::m7_optimizer::IdleOptimizer,
}

impl PipelineOrchestrator {
    /// Constructor compiling the matcher, initializing the state and tracking collections
    pub fn new() -> Self {
        Self {
            state: PipelineState {
                datasets: Vec::new(),
                literature: Vec::new(), 
                current_bytes_processed: 0,
            },
            matcher: SinglePassMatcher::compile(),
            logger: AuditLogger,
            action_history: Vec::new(),
            performance_log: Vec::new(),
            optimizer: crate::pipeline::m7_optimizer::IdleOptimizer::new(),
        }
    }

    /// Central transaction engine processing step that safely updates history logs
    pub fn record_and_apply_action(&mut self, action: IngestionAction) {
        self.action_history.push(action.clone());
        self.state = StateTransitionEngine::apply(self.state.clone(), action);
    }

    /// Upgraded chunk ingestion tracking execution speed with absolute precision
    pub fn ingest_stream_block_monitored(&mut self, source_id: &str, raw_bytes: &[u8]) {
        let byte_count = raw_bytes.len();
        self.state.current_bytes_processed += byte_count as u64;

        // Uses the dedicated structured module method to cleanly isolate metrics tuples
        let (optional_topics, metrics) = StreamProfiler::profile_block_structured(
            "Aho-Corasick State Machine",
            byte_count,
            || self.matcher.scan_raw_bytes(raw_bytes)
        );

        // Save and print out the metric data immediately
        StreamProfiler::log_metrics(&metrics);
        self.performance_log.push(metrics);

        // If validation thresholds are achieved, commit changes deterministically
        if let Some(topics) = optional_topics {
            let action = IngestionAction::BindExtractedNodes {
                source_id: source_id.to_string(),
                nodes: topics,
            };

            self.logger.on_action_triggered(action.clone());
            self.record_and_apply_action(action); // Routing through common action logic
        }
    }

    /// Legacy / Standard chunk engine ingestion entry point.
    pub fn ingest_stream_block(&mut self, source_id: &str, raw_network_bytes: &[u8]) {
        let byte_count = raw_network_bytes.len();
        self.state.current_bytes_processed += byte_count as u64;

        let optional_topics = StreamProfiler::profile_block(
            "Aho-Corasick Token Filter", 
            byte_count, 
            || self.matcher.scan_raw_bytes(raw_network_bytes)
        );

        if let Some(topics) = optional_topics {
            let action = IngestionAction::BindExtractedNodes {
                source_id: source_id.to_string(),
                nodes: topics,
            };

            self.logger.on_action_triggered(action.clone());
            self.record_and_apply_action(action);
        }
    }

    /// Commits the active memory-state down into a structured ledger file safely
    pub fn persist_to_disk(&self, disk_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let storage = LedgerStorageManager::new(disk_path);
        
        let exportable_ledger = UnifiedLedger {
            research_question: "How can early stopping criteria be optimized to prevent overfitting in edge-computing environments with strict hardware and memory limits?".to_string(),
            sources: SourcesContainer {
                datasets: self.state.datasets.clone(),
                literature: self.state.literature.clone(),
            },
        };

        storage.transactional_commit(&exportable_ledger)?;
        println!("[STORAGE] Safe transaction sync complete. Written to: {}", disk_path);
        Ok(())
    }
}
