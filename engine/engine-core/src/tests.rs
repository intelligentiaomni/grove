#[cfg(test)]
mod tests {
    use crate::pipeline::m1_state::{PipelineState, IngestionAction, StateTransitionEngine};
    use crate::pipeline::m5_replay::ReplayEngine;

    #[test]
    fn test_deterministic_state_replay() {
        // ---- PHASE 1: SIMULATE LIVE RUN ----
        let mut live_state = PipelineState {
            datasets: Vec::new(),
            literature: Vec::new(),
            current_bytes_processed: 0,
        };

        // Define historical actions in sequence
        let action_1 = IngestionAction::RegisterDatasetTarget {
            repo: "HuggingFaceFW/fineweb-edu".to_string(),
            file: "chunk_001.parquet".to_string(),
            range: "0-1000".to_string(),
        };
        
        let action_2 = IngestionAction::BindExtractedNodes {
            source_id: "pending_sha_chunk_001.parquet".to_string(),
            nodes: vec!["early-stopping".to_string(), "edge-computing".to_string()],
        };

        let action_3 = IngestionAction::CommitLiteraturePaper {
            id: "arxiv_2406.0001".to_string(),
            title: "On-Device Early Stopping Pruning".to_string(),
            nodes: vec!["ram-optimization".to_string()],
        };

        // Construct append-only transaction history log
        let history_log = vec![action_1.clone(), action_2.clone(), action_3.clone()];

        // Mutate live state sequentially
        live_state = StateTransitionEngine::apply(live_state, action_1);
        live_state = StateTransitionEngine::apply(live_state, action_2);
        live_state = StateTransitionEngine::apply(live_state, action_3);

        // ---- PHASE 2: RUN REPLAY ENGINE ----
        let replayer = ReplayEngine::new(history_log);
        let replayed_state = replayer.play_forward();

        // ---- PHASE 3: ASSERT DETERMINISTIC IDENTITY GURANTEES ----
        // Ensure dataset size matches perfectly
        assert_eq!(live_state.datasets.len(), replayed_state.datasets.len());
        assert_eq!(live_state.literature.len(), replayed_state.literature.len());

        // Ensure topics inside the dataset entries match perfectly
        assert_eq!(
            live_state.datasets[0].extracted_topics, 
            replayed_state.datasets[0].extracted_topics
        );

        // Verify partial historical slicing (inspect state directly after step 1)
        let early_state = replayer.play_up_to_index(1).unwrap();
        assert_eq!(early_state.datasets.len(), 1);
        assert_eq!(early_state.datasets[0].provenance_node_bound, false); // Topics not bound yet at step 1
        
        println!("Deterministic Replay Verification Passed. State identity is identical.");
    }
}