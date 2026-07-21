//! Shared data ingestion pipeline.
//
//! Responsible for importing external data into Grove's canonical
//! representation before validation and normalization.

pub mod api;
pub mod csv;
pub mod json;
pub mod parquet;
pub mod stream;
pub mod telemetry;

pub use api::*;
pub use csv::*;
pub use json::*;
pub use parquet::*;
pub use stream::*;
pub use telemetry::*;