use crate::hf_ingest::{DatasetSource, LiteratureSource};

#[derive(Clone, Debug)]
pub struct PipelineState {
    pub datasets: Vec<DatasetSource>,
    pub literature: Vec<LiteratureSource>,
    pub current_bytes_processed: u64,
}

// Added Clone here to resolve the E0277 and E0599 orchestrator/replay crashes
#[derive(Debug, Clone)]
pub enum IngestionAction {
    RegisterDatasetTarget { repo: String, file: String, range: String },
    BindExtractedNodes { source_id: String, nodes: Vec<String> },
    CommitLiteraturePaper { id: String, title: String, nodes: Vec<String> },
}

pub struct StateTransitionEngine;

impl StateTransitionEngine {
    pub fn apply(mut state: PipelineState, action: IngestionAction) -> PipelineState {
        match action {
            IngestionAction::RegisterDatasetTarget { repo, file, range } => {
                // Generates initial state node before byte-stream processing begins
                let dummy_hash = format!("pending_sha_{}", file);
                state.datasets.push(DatasetSource {
                    id: dummy_hash, 
                    repo, 
                    split: "train".to_string(),
                    parquet_file: file, 
                    byte_range: range,
                    extracted_topics: Vec::new(), 
                    provenance_node_bound: false,
                });
            }
            IngestionAction::BindExtractedNodes { source_id, nodes } => {
                if let Some(ds) = state.datasets.iter_mut().find(|d| d.id == source_id) {
                    ds.extracted_topics.extend(nodes);
                    ds.extracted_topics.sort();
                    ds.extracted_topics.dedup();
                    ds.provenance_node_bound = true;
                }
            }
            IngestionAction::CommitLiteraturePaper { id, title, nodes } => {
                state.literature.push(LiteratureSource {
                    id, 
                    repo: "huggingface/papers".to_string(), 
                    title,
                    authors: vec![], 
                    doi_or_url: String::new(),
                    extracted_nodes: nodes, 
                    download_timestamp: "2026-06-25T15:00:00Z".to_string(),
                });
            }
        }
        state
    }
}
