//! Time series abstractions and utilities.

/// A single observation in a time series.
#[derive(Debug, Clone)]
pub struct Observation {
    pub timestamp: String,
    pub value: f64,
}

/// A univariate time series.
#[derive(Debug, Clone, Default)]
pub struct TimeSeries {
    observations: Vec<Observation>,
}

impl TimeSeries {
    pub fn new(observations: Vec<Observation>) -> Self {
        Self { observations }
    }

    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    pub fn len(&self) -> usize {
        self.observations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }
}