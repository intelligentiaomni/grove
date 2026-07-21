//! Feature engineering for forecasting.

/// Transforms canonical records into feature vectors.
pub trait FeatureExtractor {
    type Input;
    type Output;
    type Error;

    /// Generate model-ready features.
    fn extract(
        &self,
        input: &[Self::Input],
    ) -> Result<Vec<Self::Output>, Self::Error>;
}