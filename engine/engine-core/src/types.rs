use serde::{Deserialize, Serialize};

pub type EntityId = u64;
pub type ContentHash = [u8; 32];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    OptimizationPass,
    KernelEvaluation,
    Compilation,
    Execution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionToken {
    pub parent_graph_hash: ContentHash,
    pub transition_type: TransitionType,
    pub output_artifact_hash: ContentHash,
}

impl ExecutionToken {
    pub const fn new(
        parent_graph_hash: ContentHash,
        transition_type: TransitionType,
        output_artifact_hash: ContentHash,
    ) -> Self {
        Self {
            parent_graph_hash,
            transition_type,
            output_artifact_hash,
        }
    }
}
