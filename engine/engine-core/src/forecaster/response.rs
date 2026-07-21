//! Forecast response types.
//
//! Application-level outputs produced by forecasting workflows.

/// Result returned from a forecast request.
#[derive(Debug, Clone)]
pub struct ForecastResponse {
    /// Target entity.
    pub twin_id: String,

    /// Forecasted metric.
    pub metric: String,

    /// Number of predicted steps.
    pub horizon: usize,

    /// Generated forecast values.
    pub predictions: Vec<ForecastValue>,
}

impl ForecastResponse {
    pub fn new(
        twin_id: impl Into<String>,
        metric: impl Into<String>,
        predictions: Vec<ForecastValue>,
    ) -> Self {
        let horizon = predictions.len();

        Self {
            twin_id: twin_id.into(),
            metric: metric.into(),
            horizon,
            predictions,
        }
    }
}


/// A single forecast output point.
#[derive(Debug, Clone)]
pub struct ForecastValue {
    /// Forecast timestamp.
    pub timestamp_utc: String,

    /// Predicted value.
    pub value: f64,

    /// Optional confidence score.
    pub confidence: Option<f64>,
}