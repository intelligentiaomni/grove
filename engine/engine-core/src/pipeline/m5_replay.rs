use crate::pipeline::m1_state::{PipelineState, IngestionAction, StateTransitionEngine};

pub struct ReplayEngine {
    /// An ordered, append-only log of historical transaction actions
    pub action_log: Vec<IngestionAction>,
}

impl ReplayEngine {
    pub fn new(log: Vec<IngestionAction>) -> Self {
        Self { action_log: log }
    }

    /// Backwards compatibility wrapper for state reconstruction.
    pub fn reconstruct_state(&self) -> PipelineState {
        self.play_forward()
    }

    /// Rewinds the pipeline and runs all actions from a blank slate.
    /// This returns the exact deterministic state up to the current point.
    pub fn play_forward(&self) -> PipelineState {
        let mut theoretical_state = PipelineState {
            datasets: Vec::new(),
            literature: Vec::new(),
            current_bytes_processed: 0,
        };

        for action in &self.action_log {
            // Clone the action to apply it safely without consuming the log history
            let action_clone = match action {
                IngestionAction::RegisterDatasetTarget { repo, file, range } => {
                    IngestionAction::RegisterDatasetTarget {
                        repo: repo.clone(),
                        file: file.clone(),
                        range: range.clone(),
                    }
                }
                IngestionAction::BindExtractedNodes { source_id, nodes } => {
                    IngestionAction::BindExtractedNodes {
                        source_id: source_id.clone(),
                        nodes: nodes.clone(),
                    }
                }
                IngestionAction::CommitLiteraturePaper { id, title, nodes } => {
                    IngestionAction::CommitLiteraturePaper {
                        id: id.clone(),
                        title: title.clone(),
                        nodes: nodes.clone(),
                    }
                }
            };

            // Transition state forward step-by-step
            theoretical_state = StateTransitionEngine::apply(theoretical_state, action_clone);
        }

        theoretical_state
    }

    /// Plays history forward up to a specific step index. 
    /// This allows you to inspect what your database looked like at any historical point.
    pub fn play_up_to_index(&self, index: usize) -> Option<PipelineState> {
        if index > self.action_log.len() {
            return None;
        }
        let sliced_log = self.action_log[0..index].to_vec();
        let sub_replay = ReplayEngine::new(sliced_log);
        Some(sub_replay.play_forward())
    }
}
