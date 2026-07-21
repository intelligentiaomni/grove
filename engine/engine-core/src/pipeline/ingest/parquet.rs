//! Apache Parquet ingestion backend.

use serde::de::DeserializeOwned;
use std::io::{Read, Seek};

/// Apache Parquet ingestion backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct ParquetIngestor;

impl ParquetIngestor {
    /// Deserialize a Parquet source into strongly typed records.
    ///
    /// NOTE:
    /// The implementation depends on the Parquet/Arrow library chosen by
    /// the workspace (e.g. `parquet`, `arrow`, `polars`).
    pub fn ingest<T, R>(&self, reader: R) -> Result<Vec<T>, IngestError>
    where
        T: DeserializeOwned,
        R: Read + Seek,
    {
        // TODO:
        // 1. Open Parquet reader.
        // 2. Read record batches.
        // 3. Convert rows into `T`.
        // 4. Return Vec<T>.

        todo!("Implement Parquet deserialization")
    }
}

/// Placeholder until the pipeline has a unified error type.
#[derive(Debug)]
pub enum IngestError {
    Io(std::io::Error),
    Decode(String),
}