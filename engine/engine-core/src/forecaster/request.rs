//! Forecast request types.
//
//! Application-level inputs for forecast operations.

/// Request to generate a forecast.
#[derive(Debug, Clone)]
pub struct ForecastRequest {
    /// Target entity being forecasted.
    pub twin_id: String,

    /// Metric or signal to predict.
    pub metric: String,

    /// Number of future prediction steps.
    pub horizon: usize,

    /// Optional preferred model identifier.
    pub model_id: Option<String>,
}

impl ForecastRequest {
    /// Create a new forecast request.
    pub fn new(
        twin_id: impl Into<String>,
        metric: impl Into<String>,
        horizon: usize,
    ) -> Self {
        Self {
            twin_id: twin_id.into(),
            metric: metric.into(),
            horizon,
            model_id: None,
        }
    }

    /// Attach a preferred model.
    pub fn with_model(
        mut self,
        model_id: impl Into<String>,
    ) -> Self {
        self.model_id = Some(model_id.into());
        self
    }
}