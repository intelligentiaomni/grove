use crate::EpistemicExtractionService;
use engine_core::CorrespondenceGraph;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::Arc;
use tiktoken_rs::{cl100k_base, CoreBPE};

pub const DEFAULT_LOCAL_TOKEN_LIMIT: usize = 40_000;
pub const DEFAULT_SERVERLESS_TOKEN_LIMIT: usize = 120_000;
pub const FALLBACK_CHUNK_WORD_LIMIT: usize = 24_999;
pub const LOCAL_OLLAMA_MODEL: &str = "epistemic-3090";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceRoute {
    LocalOllama {
        model: String,
        token_weight: usize,
    },
    Serverless {
        endpoint: String,
        token_weight: usize,
    },
}

#[derive(Debug)]
pub enum RouterError {
    Tokenizer(String),
    MissingServerlessEndpoint,
    PayloadTooLarge {
        token_weight: usize,
        max_tokens: usize,
    },
    Provider(Box<dyn Error + Send + Sync>),
}

impl Display for RouterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Tokenizer(err) => write!(f, "tokenizer initialization failed: {err}"),
            Self::MissingServerlessEndpoint => f.write_str("SERVERLESS_ENDPOINT is not configured"),
            Self::PayloadTooLarge {
                token_weight,
                max_tokens,
            } => write!(
                f,
                "payload token weight {token_weight} exceeds router maximum {max_tokens}"
            ),
            Self::Provider(err) => write!(f, "inference provider failed: {err}"),
        }
    }
}

impl Error for RouterError {}

#[derive(Clone)]
pub struct DynamicInferenceRouter {
    tokenizer: Arc<CoreBPE>,
    client: reqwest::Client,
    ollama_endpoint: String,
    serverless_endpoint: Option<String>,
    local_token_limit: usize,
    serverless_token_limit: usize,
}

#[derive(Debug, Serialize)]
struct ServerlessRequest<'a> {
    text: &'a str,
    token_weight: usize,
}

impl DynamicInferenceRouter {
    pub fn from_env() -> Result<Self, RouterError> {
        let ollama_endpoint = std::env::var("OLLAMA_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:11434/api/generate".to_string());
        let serverless_endpoint = std::env::var("SERVERLESS_ENDPOINT").ok();
        Self::new(
            ollama_endpoint,
            serverless_endpoint,
            DEFAULT_LOCAL_TOKEN_LIMIT,
            DEFAULT_SERVERLESS_TOKEN_LIMIT,
        )
    }

    pub fn new(
        ollama_endpoint: impl Into<String>,
        serverless_endpoint: Option<String>,
        local_token_limit: usize,
        serverless_token_limit: usize,
    ) -> Result<Self, RouterError> {
        let tokenizer =
            Arc::new(cl100k_base().map_err(|err| RouterError::Tokenizer(err.to_string()))?);
        Ok(Self {
            tokenizer,
            client: reqwest::Client::new(),
            ollama_endpoint: ollama_endpoint.into(),
            serverless_endpoint,
            local_token_limit,
            serverless_token_limit,
        })
    }

    pub fn token_weight(&self, text: &str) -> usize {
        self.tokenizer.encode_with_special_tokens(text).len()
    }

    pub fn route_for(&self, text: &str) -> Result<InferenceRoute, RouterError> {
        let token_weight = self.token_weight(text);

        if token_weight < self.local_token_limit {
            return Ok(InferenceRoute::LocalOllama {
                model: LOCAL_OLLAMA_MODEL.to_string(),
                token_weight,
            });
        }

        if token_weight <= self.serverless_token_limit {
            let endpoint = self
                .serverless_endpoint
                .clone()
                .ok_or(RouterError::MissingServerlessEndpoint)?;
            return Ok(InferenceRoute::Serverless {
                endpoint,
                token_weight,
            });
        }

        Err(RouterError::PayloadTooLarge {
            token_weight,
            max_tokens: self.serverless_token_limit,
        })
    }

    pub async fn process_text_to_graph(
        &self,
        text: &str,
    ) -> Result<CorrespondenceGraph, RouterError> {
        match self.route_for(text)? {
            InferenceRoute::LocalOllama { .. } => {
                EpistemicExtractionService::new(&self.ollama_endpoint, LOCAL_OLLAMA_MODEL)
                    .process_text_to_graph(text)
                    .await
                    .map_err(RouterError::Provider)
            }
            InferenceRoute::Serverless {
                endpoint,
                token_weight,
            } => match self
                .process_text_to_graph_explicit(&endpoint, text, token_weight)
                .await
            {
                Ok(graph) => Ok(graph),
                Err(err) if is_network_resilience_error(&err) => {
                    eprintln!(
                        "system warning: serverless inference failed ({err}); invoking local compression fallback"
                    );
                    self.execute_local_compression_fallback(text).await
                }
                Err(err) => Err(RouterError::Provider(Box::new(err))),
            },
        }
    }

    pub async fn process_text_to_graph_explicit(
        &self,
        endpoint: &str,
        text: &str,
        token_weight: usize,
    ) -> Result<CorrespondenceGraph, reqwest::Error> {
        self.client
            .post(endpoint)
            .json(&ServerlessRequest { text, token_weight })
            .send()
            .await?
            .error_for_status()?
            .json::<CorrespondenceGraph>()
            .await
    }

    pub async fn execute_local_compression_fallback(
        &self,
        text: &str,
    ) -> Result<CorrespondenceGraph, RouterError> {
        let local = EpistemicExtractionService::new(&self.ollama_endpoint, LOCAL_OLLAMA_MODEL);
        let mut nodes = Vec::new();

        for chunk in chunk_text_under_word_limit(text, FALLBACK_CHUNK_WORD_LIMIT) {
            let graph = local
                .process_text_to_graph(&chunk)
                .await
                .map_err(RouterError::Provider)?;
            nodes.extend(graph.nodes);
        }

        Ok(CorrespondenceGraph::new(nodes))
    }
}

fn is_network_resilience_error(err: &reqwest::Error) -> bool {
    err.is_status()
        || err.is_timeout()
        || err.is_connect()
        || err
            .to_string()
            .to_ascii_lowercase()
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .any(|token| matches!(token, "dns" | "resolve" | "resolution"))
}

fn chunk_text_under_word_limit(text: &str, word_limit: usize) -> Vec<String> {
    if word_limit == 0 {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_words = 0_usize;

    for word in text.split_whitespace() {
        if current_words == word_limit {
            chunks.push(current);
            current = String::new();
            current_words = 0;
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
        current_words += 1;
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::{
        chunk_text_under_word_limit, DynamicInferenceRouter, InferenceRoute, RouterError,
        DEFAULT_LOCAL_TOKEN_LIMIT, DEFAULT_SERVERLESS_TOKEN_LIMIT, FALLBACK_CHUNK_WORD_LIMIT,
        LOCAL_OLLAMA_MODEL,
    };

    fn router(local_limit: usize, serverless_limit: usize) -> DynamicInferenceRouter {
        DynamicInferenceRouter::new(
            "http://127.0.0.1:11434/api/generate",
            Some("https://serverless.example/extract".to_string()),
            local_limit,
            serverless_limit,
        )
        .expect("tokenizer should initialize")
    }

    #[test]
    fn computes_exact_bpe_weight_for_known_cl100k_sample() {
        let router = router(DEFAULT_LOCAL_TOKEN_LIMIT, DEFAULT_SERVERLESS_TOKEN_LIMIT);

        assert_eq!(router.token_weight("hello world"), 2);
    }

    #[test]
    fn routes_below_threshold_to_epistemic_3090() {
        let router = router(4, 8);

        let route = router
            .route_for("hello world")
            .expect("route should resolve");

        assert_eq!(
            route,
            InferenceRoute::LocalOllama {
                model: LOCAL_OLLAMA_MODEL.to_string(),
                token_weight: 2,
            }
        );
    }

    #[test]
    fn routes_extended_context_to_serverless_endpoint() {
        let router = router(1, 8);

        let route = router
            .route_for("hello world")
            .expect("route should resolve");

        assert_eq!(
            route,
            InferenceRoute::Serverless {
                endpoint: "https://serverless.example/extract".to_string(),
                token_weight: 2,
            }
        );
    }

    #[test]
    fn rejects_payloads_above_serverless_ceiling() {
        let router = router(1, 1);

        let err = router
            .route_for("hello world")
            .expect_err("two-token payload exceeds one-token ceiling");

        assert!(matches!(
            err,
            RouterError::PayloadTooLarge {
                token_weight: 2,
                max_tokens: 1
            }
        ));
    }

    #[test]
    fn requires_serverless_endpoint_for_cloud_route() {
        let router = DynamicInferenceRouter::new("http://127.0.0.1:11434/api/generate", None, 1, 8)
            .expect("tokenizer should initialize");

        let err = router
            .route_for("hello world")
            .expect_err("cloud route needs endpoint");

        assert!(matches!(err, RouterError::MissingServerlessEndpoint));
    }

    #[test]
    fn fallback_chunker_keeps_chunks_under_word_ceiling() {
        let payload = (0..50_001)
            .map(|idx| format!("word{idx}"))
            .collect::<Vec<_>>()
            .join(" ");

        let chunks = chunk_text_under_word_limit(&payload, FALLBACK_CHUNK_WORD_LIMIT);

        assert_eq!(chunks.len(), 3);
        assert!(chunks
            .iter()
            .all(|chunk| chunk.split_whitespace().count() < 25_000));
        assert_eq!(
            chunks
                .iter()
                .map(|chunk| chunk.split_whitespace().count())
                .sum::<usize>(),
            50_001
        );
    }
}
