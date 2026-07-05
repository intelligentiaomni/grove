use axum::body::Bytes;
use engine_core::{GraphEdgeRecord, GraphLineageController, GraphLineageError, GraphNodeRecord};
use engine_ml::DynamicInferenceRouter;
use parquet::errors::ParquetError;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::RowAccessor;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs;
use std::path::Path;

const DEFAULT_TEXT_COLUMN: &str = "text";
const LINEAGE_DB_PATH: &str = "kernel_store/lineage/hf_ingest.sqlite";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfIngestRequest {
    pub parquet_url: String,
    pub text_column: Option<String>,
    pub max_documents: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HfIngestReport {
    pub documents_processed: usize,
    pub topic_nodes_persisted: usize,
    pub lineage_edges_persisted: usize,
}

#[derive(Debug)]
pub enum HfIngestError {
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    Network(reqwest::Error),
    Parquet(ParquetError),
    MissingTextColumn(String),
    Join(tokio::task::JoinError),
    Lineage(GraphLineageError),
    Io(std::io::Error),
    Router(engine_ml::RouterError),
    Serialize(serde_json::Error),
}

impl Display for HfIngestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::InvalidHeader(err) => write!(f, "invalid Hugging Face auth header: {err}"),
            Self::Network(err) => write!(f, "Hugging Face network request failed: {err}"),
            Self::Parquet(err) => write!(f, "Parquet decode failed: {err}"),
            Self::MissingTextColumn(column) => write!(f, "Parquet text column not found: {column}"),
            Self::Join(err) => write!(f, "Parquet decode worker failed: {err}"),
            Self::Lineage(err) => write!(f, "lineage persistence failed: {err}"),
            Self::Io(err) => write!(f, "ingest file operation failed: {err}"),
            Self::Router(err) => write!(f, "dynamic inference routing failed: {err}"),
            Self::Serialize(err) => write!(f, "topic serialization failed: {err}"),
        }
    }
}

impl std::error::Error for HfIngestError {}

impl From<reqwest::header::InvalidHeaderValue> for HfIngestError {
    fn from(value: reqwest::header::InvalidHeaderValue) -> Self {
        Self::InvalidHeader(value)
    }
}

impl From<reqwest::Error> for HfIngestError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<ParquetError> for HfIngestError {
    fn from(value: ParquetError) -> Self {
        Self::Parquet(value)
    }
}

impl From<GraphLineageError> for HfIngestError {
    fn from(value: GraphLineageError) -> Self {
        Self::Lineage(value)
    }
}

impl From<std::io::Error> for HfIngestError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<engine_ml::RouterError> for HfIngestError {
    fn from(value: engine_ml::RouterError) -> Self {
        Self::Router(value)
    }
}

impl From<serde_json::Error> for HfIngestError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialize(value)
    }
}

#[derive(Clone)]
pub struct EliteDatasetIngester {
    client: reqwest::Client,
}

impl EliteDatasetIngester {
    pub fn from_env() -> Result<Self, HfIngestError> {
        let mut headers = HeaderMap::new();
        if let Ok(token) = std::env::var("HF_TOKEN") {
            if !token.trim().is_empty() {
                let value = HeaderValue::from_str(&format!("Bearer {}", token.trim()))?;
                headers.insert(AUTHORIZATION, value);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Self { client })
    }

    pub async fn fetch_text_blocks(
        &self,
        parquet_url: &str,
        text_column: &str,
        max_documents: Option<usize>,
    ) -> Result<Vec<String>, HfIngestError> {
        let bytes = self
            .client
            .get(parquet_url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec();
        let text_column = text_column.to_string();

        tokio::task::spawn_blocking(move || {
            decode_parquet_text_blocks(&bytes, &text_column, max_documents)
        })
        .await
        .map_err(HfIngestError::Join)?
    }
}

pub async fn ingest_hf_dataset_into_lineage(
    request: HfIngestRequest,
) -> Result<HfIngestReport, HfIngestError> {
    let text_column = request
        .text_column
        .as_deref()
        .unwrap_or(DEFAULT_TEXT_COLUMN)
        .to_string();
    let ingester = EliteDatasetIngester::from_env()?;
    let documents = ingester
        .fetch_text_blocks(&request.parquet_url, &text_column, request.max_documents)
        .await?;
    let router = DynamicInferenceRouter::from_env()?;

    fs::create_dir_all(
        Path::new(LINEAGE_DB_PATH)
            .parent()
            .expect("lineage path parent"),
    )?;
    let lineage = GraphLineageController::open(LINEAGE_DB_PATH)?;
    persist_documents_to_lineage(&documents, &router, &lineage).await
}

pub fn decode_parquet_text_blocks(
    bytes: &[u8],
    text_column: &str,
    max_documents: Option<usize>,
) -> Result<Vec<String>, HfIngestError> {
    let reader = SerializedFileReader::new(Bytes::copy_from_slice(bytes))?;
    let schema = reader.metadata().file_metadata().schema_descr();
    let column_idx = schema
        .columns()
        .iter()
        .position(|column| column.name() == text_column)
        .ok_or_else(|| HfIngestError::MissingTextColumn(text_column.to_string()))?;
    let row_iter = reader.get_row_iter(None)?;
    let limit = max_documents.unwrap_or(usize::MAX);
    let mut blocks = Vec::new();

    for row in row_iter.take(limit) {
        let row = row?;
        let text = row.get_string(column_idx)?;
        if !text.trim().is_empty() {
            blocks.push(text.to_string());
        }
    }

    Ok(blocks)
}

async fn persist_documents_to_lineage(
    documents: &[String],
    router: &DynamicInferenceRouter,
    lineage: &GraphLineageController,
) -> Result<HfIngestReport, HfIngestError> {
    let mut topic_nodes_persisted = 0_usize;
    let mut lineage_edges_persisted = 0_usize;

    for document in documents {
        let now = chrono::Utc::now().timestamp_millis();
        let document_hash = stable_hash(document.as_bytes());
        let document_node_hash = format!("hf-document:{document_hash}");
        lineage.insert_node(&GraphNodeRecord {
            node_hash: document_node_hash.clone(),
            payload_hash: document_hash,
            kind: "hf_text_document".to_string(),
            created_at_ms: now,
        })?;

        let graph = router.process_text_to_graph(document).await?;
        for topic in graph.nodes {
            let topic_bytes = serde_json::to_vec(&topic)?;
            let topic_hash = stable_hash(&topic_bytes);
            let topic_node_hash = format!("topic-switch:{topic_hash}");
            lineage.insert_node(&GraphNodeRecord {
                node_hash: topic_node_hash.clone(),
                payload_hash: topic_hash,
                kind: "topic_switch".to_string(),
                created_at_ms: now,
            })?;
            lineage.insert_edge(&GraphEdgeRecord {
                parent_hash: document_node_hash.clone(),
                child_hash: topic_node_hash,
                relation: "maps_to_topic_switch".to_string(),
                created_at_ms: now,
            })?;
            topic_nodes_persisted += 1;
            lineage_edges_persisted += 1;
        }
    }

    Ok(HfIngestReport {
        documents_processed: documents.len(),
        topic_nodes_persisted,
        lineage_edges_persisted,
    })
}

fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{decode_parquet_text_blocks, stable_hash, HfIngestError};

    #[test]
    fn stable_hash_is_deterministic() {
        assert_eq!(stable_hash(b"elite corpus"), stable_hash(b"elite corpus"));
        assert_ne!(stable_hash(b"elite corpus"), stable_hash(b"other corpus"));
    }

    #[test]
    fn invalid_parquet_bytes_return_decode_error() {
        let err = decode_parquet_text_blocks(b"not parquet", "text", Some(1))
            .expect_err("invalid parquet should fail");

        assert!(matches!(err, HfIngestError::Parquet(_)));
    }
}
