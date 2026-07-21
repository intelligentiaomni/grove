//! Forecast model abstractions.

/// Generic forecasting model.
pub trait ForecastModel {
    type Input;
    type Output;
    type Error;

    /// Train the model.
    fn fit(&mut self, training_data: &[Self::Input]) -> Result<(), Self::Error>;

    /// Produce a forecast.
    fn predict(&self, input: &Self::Input)
        -> Result<Self::Output, Self::Error>;
}