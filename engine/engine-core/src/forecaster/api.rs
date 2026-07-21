//! Forecast application API.
//
//! Public interface for requesting forecasts.

use super::{
    request::ForecastRequest,
    response::ForecastResponse,
    service::ForecasterService,
};

/// Public forecaster API.
#[derive(Debug)]
pub struct ForecasterApi {
    service: ForecasterService,
}

impl ForecasterApi {
    /// Create a new forecasting API.
    pub fn new(
        service: ForecasterService,
    ) -> Self {
        Self {
            service,
        }
    }

    /// Generate a forecast.
    pub fn forecast(
        &self,
        request: ForecastRequest,
    ) -> Result<ForecastResponse, ForecastApiError> {
        self.validate(&request)?;

        self.service
            .forecast(request)
            .map_err(ForecastApiError::Execution)
    }

    fn validate(
        &self,
        request: &ForecastRequest,
    ) -> Result<(), ForecastApiError> {
        if request.twin_id.is_empty() {
            return Err(
                ForecastApiError::InvalidRequest(
                    "missing twin id".into(),
                ),
            );
        }

        if request.metric.is_empty() {
            return Err(
                ForecastApiError::InvalidRequest(
                    "missing metric".into(),
                ),
            );
        }

        if request.horizon == 0 {
            return Err(
                ForecastApiError::InvalidRequest(
                    "forecast horizon must be greater than zero".into(),
                ),
            );
        }

        Ok(())
    }
}