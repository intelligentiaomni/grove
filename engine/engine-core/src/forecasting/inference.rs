//! Forecast inference.

use super::model::ForecastModel;

/// Executes inference using trained forecasting models.
#[derive(Debug, Default)]
pub struct ForecastInference;

impl ForecastInference {
    /// Generate a forecast for the given prediction horizon.
    pub fn forecast<M>(
        &self,
        model: &M,
        input: &M::Input,
        horizon: usize,
    ) -> Result<M::Forecast, M::Error>
    where
        M: ForecastModel,
    {
        model.forecast(input, horizon)
    }
}