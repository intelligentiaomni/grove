//! Forecaster application layer.
//
//! Provides the high-level forecasting interface consumed by
//! execution tasks, APIs, and downstream Grove modules.

pub mod api;
pub mod request;
pub mod response;
pub mod service;

// Public application API.
pub use api::{
    ForecastApiError,
    ForecasterApi,
};

pub use request::ForecastRequest;

pub use response::{
    ForecastResponse,
    ForecastValue,
};

pub use service::ForecasterService;