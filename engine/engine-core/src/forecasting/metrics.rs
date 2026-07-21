//! Forecast evaluation metrics.

/// Computes an evaluation metric from observed and predicted values.
pub trait ForecastMetric {
    /// Human-readable metric name.
    fn name(&self) -> &'static str;

    /// Compute the metric value.
    fn compute(
        &self,
        actual: &[f64],
        predicted: &[f64],
    ) -> Result<f64, MetricError>;
}

#[derive(Debug)]
pub enum MetricError {
    LengthMismatch,
    EmptyInput,
}