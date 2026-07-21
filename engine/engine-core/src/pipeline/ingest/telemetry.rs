//! Telemetry ingestion backend.

use crate::telemetry::TelemetryPayload;

/// Live telemetry ingestion backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct TelemetryIngestor;

impl TelemetryIngestor {
    /// Ingest a single telemetry payload.
    pub fn ingest(
        &self,
        payload: TelemetryPayload,
    ) -> Result<TelemetryPayload, TelemetryError> {
        // Future work:
        // - Basic schema validation
        // - Timestamp sanity checks
        // - Unit normalization
        // - Attach source metadata

        Ok(payload)
    }

    /// Ingest a batch of telemetry payloads.
    pub fn ingest_batch(
        &self,
        payloads: Vec<TelemetryPayload>,
    ) -> Result<Vec<TelemetryPayload>, TelemetryError> {
        Ok(payloads)
    }
}

#[derive(Debug)]
pub enum TelemetryError {
    InvalidPayload(String),
}