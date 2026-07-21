//! CSV ingestion backend.

use std::io::Read;

use csv::ReaderBuilder;
use serde::de::DeserializeOwned;

/// CSV ingestion backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct CsvIngestor;

impl CsvIngestor {
    /// Deserialize CSV rows into strongly typed records.
    pub fn ingest<T, R>(&self, reader: R) -> Result<Vec<T>, csv::Error>
    where
        T: DeserializeOwned,
        R: Read,
    {
        let mut csv = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        csv.deserialize().collect()
    }
}