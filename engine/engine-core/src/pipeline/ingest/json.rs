//! JSON ingestion backend.

use serde::de::DeserializeOwned;
use std::io::Read;

/// JSON ingestion backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonIngestor;

impl JsonIngestor {
    /// Deserialize JSON into a strongly typed value.
    ///
    /// Supports:
    /// - A single JSON object
    /// - An array of JSON objects
    pub fn ingest<T, R>(&self, reader: R) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
        R: Read,
    {
        serde_json::from_reader(reader)
    }
}