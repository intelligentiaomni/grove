//! Forecast model training.

use super::model::ForecastModel;

/// Generic trainer for forecast models.
#[derive(Debug, Default)]
pub struct ForecastTrainer;

impl ForecastTrainer {
    /// Train a forecasting model.
    pub fn train<M>(
        &self,
        model: &mut M,
        training_data: &[M::Input],
    ) -> Result<(), M::Error>
    where
        M: ForecastModel,
    {
        model.fit(training_data)
    }
}