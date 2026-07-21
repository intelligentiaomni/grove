//! Streaming ingestion abstractions.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

/// Generic stream ingestor.
///
/// Wraps any asynchronous stream of records so it can participate in the
/// shared ingestion pipeline.
#[derive(Debug)]
pub struct StreamIngestor<S> {
    inner: S,
}

impl<S> StreamIngestor<S> {
    pub fn new(stream: S) -> Self {
        Self { inner: stream }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S> Stream for StreamIngestor<S>
where
    S: Stream + Unpin,
{
    type Item = S::Item;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}