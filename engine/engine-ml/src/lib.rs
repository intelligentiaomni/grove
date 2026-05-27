//! engine-ml: small adapter to load ONNX models via ORT (native libs required).
pub mod dynamic_router;
pub mod inference;
pub mod prefix_control;
pub mod scheduler;
pub mod scientific_reasoner;

pub use dynamic_router::{DynamicInferenceRouter, InferenceRoute, RouterError};
pub use engine_kernel::{
    register_token_parsing_matrix_weights, KernelState, NpuTaskFrame, Quantization,
};
pub use inference::{process_text_to_graph, EpistemicExtractionService};
pub use prefix_control::{PrefixCacheController, PrefixCacheEntry};
pub use scheduler::ContinuousResearchScheduler;
pub use scientific_reasoner::{
    HybridScientificReasoner, ScientificPipelineRequest, ScientificValidationReport,
};

pub fn ml_version() -> &'static str {
    "engine-ml v0.1.0"
}

// Example API — left as a stub to be implemented when ORT / tch chosen
pub fn infer_stub() -> String {
    "ml infer stub (install onnxruntime and enable crate)".to_string()
}
