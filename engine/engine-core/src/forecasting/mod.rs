//! Forecasting layer.
//
//! Responsible for training, evaluating, and executing predictive models
//! using canonical records produced by the shared data pipeline.

pub mod features;
pub mod inference;
pub mod metrics;
pub mod model;
pub mod timeseries;
pub mod trainer;
pub mod uncertainty;

// Public API
pub use features::*;
pub use inference::*;
pub use metrics::*;
pub use model::*;
pub use timeseries::*;
pub use trainer::*;
pub use uncertainty::*;