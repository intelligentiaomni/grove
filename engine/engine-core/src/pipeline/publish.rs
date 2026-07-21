//! Pipeline publication stage.

/// A consumer of pipeline records.
pub trait Publisher<T> {
    type Error;

    /// Publish a processed record.
    fn publish(&self, record: T) -> Result<(), Self::Error>;
}

/// Publishes records to one or more downstream consumers.
#[derive(Debug, Default)]
pub struct PipelinePublisher<P> {
    publishers: Vec<P>,
}

impl<P> PipelinePublisher<P> {
    pub fn new() -> Self {
        Self {
            publishers: Vec::new(),
        }
    }

    pub fn register(&mut self, publisher: P) {
        self.publishers.push(publisher);
    }
}

impl<T, P> PipelinePublisher<P>
where
    P: Publisher<T>,
    T: Clone,
{
    pub fn publish(&self, record: T) -> Result<(), P::Error> {
        for publisher in &self.publishers {
            publisher.publish(record.clone())?;
        }

        Ok(())
    }
}