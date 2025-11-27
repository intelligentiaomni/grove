//! engine-ml: small adapter to load ONNX models via ORT (native libs required).
pub fn ml_version() -> &'static str { "engine-ml v0.1.0" }

// Example API — left as a stub to be implemented when ORT / tch chosen
pub fn infer_stub() -> String {
    "ml infer stub (install onnxruntime and enable crate)".to_string()
}
