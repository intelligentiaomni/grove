//! HTTP API ingestion backend.

use serde::de::DeserializeOwned;

/// HTTP API ingestion backend.
#[derive(Debug, Default, Clone)]
pub struct ApiIngestor {
    client: reqwest::Client,
}

impl ApiIngestor {
    /// Create a new API ingestor.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch and deserialize JSON from an HTTP endpoint.
    pub async fn ingest<T>(&self, url: &str) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await
    }
}