//! Forecaster application service.
//
//! Orchestrates forecasting workflows using the forecasting subsystem.

use crate::forecasting::{
    ForecastInference,
};

use super::{
    request::ForecastRequest,
    response::{
        ForecastResponse,
        ForecastValue,
    },
};

/// Application service for forecasting operations.
#[derive(Debug)]
pub struct ForecasterService {
    inference: ForecastInference,
}

impl ForecasterService {
    /// Create a new forecaster service.
    pub fn new(
        inference: ForecastInference,
    ) -> Self {
        Self {
            inference,
        }
    }

    /// Execute a forecasting request.
    pub fn forecast(
        &self,
        request: ForecastRequest,
    ) -> Result<ForecastResponse, ForecastServiceError> {
        let predictions = self.run_inference(&request)?;

        Ok(ForecastResponse {
            twin_id: request.twin_id,
            metric: request.metric,
            horizon: request.horizon,
            predictions,
        })
    }

    fn run_inference(
        &self,
        request: &ForecastRequest,
    ) -> Result<Vec<ForecastValue>, ForecastServiceError> {

        // Future workflow:
        //
        // 1. Load historical observations
        // 2. Convert to TimeSeries
        // 3. Extract features
        // 4. Select ForecastModel
        // 5. Execute inference
        // 6. Estimate uncertainty
        //
        // Current placeholder.

        let _ = &self.inference;

        Ok(Vec::with_capacity(request.horizon))
    }
}