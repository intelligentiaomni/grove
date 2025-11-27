//! inference.rs
//!
//! Generic inference traits + a simple runtime wrapper.
//! Backends can implement `ModelBackend` and plug into the engine.

use anyhow::Result;

/// A generic machine-learning inference backend.
///
/// Examples:
/// - ONNX runtime (native)
/// - WebGPU/WASM inference
/// - GGML backend
/// - Candle transformer models
pub trait ModelBackend: Send + Sync {
    /// Load a model from bytes
    fn load(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;

    /// Run inference on the given input tensor(s)
    fn infer(&self, input: &InferenceInput) -> Result<InferenceOutput>;
}

/// Simple structured input
#[derive(Debug, Clone)]
pub struct InferenceInput {
    pub data: Vec<f32>,
    pub dims: Vec<usize>,
}

impl InferenceInput {
    pub fn new(data: Vec<f32>, dims: Vec<usize>) -> Self {
        Self { data, dims }
    }
}

/// Simple structured output
#[derive(Debug, Clone)]
pub struct InferenceOutput {
    pub data: Vec<f32>,
    pub dims: Vec<usize>,
}

impl InferenceOutput {
    pub fn new(data: Vec<f32>, dims: Vec<usize>) -> Self {
        Self { data, dims }
    }
}

/// High-level runtime wrapper used by other engine crates.
/// This allows you to swap backends at compile-time or run-time.
pub struct InferenceRuntime<B: ModelBackend> {
    backend: B,
}

impl<B: ModelBackend> InferenceRuntime<B> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let backend = B::load(bytes)?;
        Ok(Self { backend })
    }

    pub fn infer(&self, input: &InferenceInput) -> Result<InferenceOutput> {
        self.backend.infer(input)
    }
}

