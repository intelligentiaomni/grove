//! inference.rs
//!
//! Generic inference traits + a simple runtime wrapper.
//! Backends can implement `ModelBackend` and plug into the engine.

use anyhow::Result;
use engine_core::CorrespondenceGraph;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::OnceLock;

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

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

static DEFAULT_EXTRACTION_SERVICE: OnceLock<EpistemicExtractionService> = OnceLock::new();

#[derive(Clone)]
pub struct EpistemicExtractionService {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl EpistemicExtractionService {
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into(),
            model: model.into(),
        }
    }

    pub fn from_env() -> Self {
        let endpoint = std::env::var("OLLAMA_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:11434/api/generate".to_string());
        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.1".to_string());
        Self::new(endpoint, model)
    }

    pub async fn process_text_to_graph(
        &self,
        text: &str,
    ) -> std::result::Result<CorrespondenceGraph, Box<dyn Error + Send + Sync>> {
        let prompt = build_epistemic_prompt(text);
        let request = OllamaGenerateRequest {
            model: self.model.as_str(),
            prompt,
            stream: false,
        };

        let response = self
            .client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json::<OllamaGenerateResponse>()
            .await?;

        parse_graph_response(&response.response)
    }
}

pub async fn process_text_to_graph(
    text: &str,
) -> std::result::Result<CorrespondenceGraph, Box<dyn Error + Send + Sync>> {
    DEFAULT_EXTRACTION_SERVICE
        .get_or_init(EpistemicExtractionService::from_env)
        .process_text_to_graph(text)
        .await
}

fn build_epistemic_prompt(text: &str) -> String {
    let mut prompt = String::with_capacity(text.len() + 768);
    prompt.push_str(
        r#"Extract an epistemic correspondence graph from the text.

Return only valid JSON matching this exact schema:
{"nodes":[{"what":"core technical concept or business topic shift","who":["actor or entity"],"next_step":"actionable resolution or task item"}]}

Rules:
- Preserve topic order from the source text.
- Keep "what" concise and concrete.
- Put people, teams, products, systems, or organizations in "who".
- Use an empty array for "who" when no actor is present.
- Use an empty string for "next_step" when no action is implied.
- Do not include markdown, commentary, or fields outside the schema.

Text:
"#,
    );
    prompt.push_str(text);
    prompt
}

fn parse_graph_response(
    response: &str,
) -> std::result::Result<CorrespondenceGraph, Box<dyn Error + Send + Sync>> {
    let trimmed = response.trim();
    match serde_json::from_str::<CorrespondenceGraph>(trimmed) {
        Ok(graph) => Ok(graph),
        Err(_) => {
            let start = trimmed
                .find('{')
                .ok_or("Ollama response did not contain JSON")?;
            let end = trimmed
                .rfind('}')
                .ok_or("Ollama response did not contain JSON")?;
            let graph = serde_json::from_str::<CorrespondenceGraph>(&trimmed[start..=end])?;
            Ok(graph)
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{build_epistemic_prompt, parse_graph_response};

    #[test]
    fn autonomous_mock_parses_correspondence_graph() {
        let response = r#"
        {
          "nodes": [
            {
              "what": "Roll out epistemic extraction endpoint",
              "who": ["engine-ml", "engine-server"],
              "next_step": "Wire raw text payloads to the inference service"
            }
          ]
        }
        "#;

        let graph = parse_graph_response(response).expect("mock graph should deserialize");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(
            graph.nodes[0].what,
            "Roll out epistemic extraction endpoint"
        );
        assert_eq!(graph.nodes[0].who, ["engine-ml", "engine-server"]);
        assert_eq!(
            graph.nodes[0].next_step,
            "Wire raw text payloads to the inference service"
        );
    }

    #[test]
    fn autonomous_mock_parses_json_from_model_chatter() {
        let response = r#"Here is the graph:
        {"nodes":[{"what":"Budget handoff","who":["Ops"],"next_step":"Confirm owner"}]}
        "#;

        let graph = parse_graph_response(response).expect("embedded JSON should deserialize");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].what, "Budget handoff");
        assert_eq!(graph.nodes[0].who, ["Ops"]);
        assert_eq!(graph.nodes[0].next_step, "Confirm owner");
    }

    #[test]
    fn prompt_contains_source_text_without_cloning_contract_changes() {
        let source = "Alice asked Bob to split the dashboard follow-up from the Rust pipeline.";
        let prompt = build_epistemic_prompt(source);

        assert!(prompt.contains("Return only valid JSON"));
        assert!(prompt.contains(source));
    }
}
