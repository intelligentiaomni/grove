//! Forecast uncertainty estimation.

/// Represents uncertainty around a forecast value.
#[derive(Debug, Clone)]
pub struct PredictionInterval {
    /// Lower bound of the interval.
    pub lower: f64,

    /// Upper bound of the interval.
    pub upper: f64,

    /// Confidence level (e.g. 0.95 for 95%).
    pub confidence: f64,
}

/// Forecast result with uncertainty information.
#[derive(Debug, Clone)]
pub struct UncertainForecast<T> {
    pub forecast: T,
    pub intervals: Vec<PredictionInterval>,
}