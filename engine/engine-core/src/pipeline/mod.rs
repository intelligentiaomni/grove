//! Shared data pipeline.
//
//! The pipeline is responsible for moving external data into Grove's
//! canonical representation through a sequence of processing stages:
//
//! ingest
//!   ↓
//! validate
//!   ↓
//! normalize
//!   ↓
//! provenance
//!   ↓
//! publish

pub mod ingest;
pub mod normalize;
pub mod provenance;
pub mod publish;
pub mod validate;

// Re-export common pipeline interfaces.
pub use ingest::*;
pub use normalize::*;
pub use provenance::*;
pub use publish::*;
pub use validate::*;