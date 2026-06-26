use std::time::{Duration, Instant};

/// Structured telemetry payload summarizing a single streaming state transition
#[derive(Debug, Clone)]
pub struct TransitionMetrics {
    pub subsystem_label: String, // Upgraded to String for safer lifetime handling across systems
    pub bytes_processed: usize,
    pub execution_time: Duration,
    pub throughput_mb_per_sec: f64,
}

pub struct StreamProfiler;

impl StreamProfiler {
    /// Profiles a closure operation, logs telemetry to stdout, and preserves 
    /// the backward-compatible return signature for the orchestration pipeline.
    pub fn profile_block<F, T>(label: &str, byte_count: usize, mut operation: F) -> T
    where
        F: FnMut() -> T,
    {
        // 1. Capture nanosecond-precision execution metrics via the structured helper
        let (result, metrics) = Self::profile_block_structured(label, byte_count, &mut operation);

        // 2. Mirror the immediate console printout feature of the older version
        Self::log_metrics(&metrics);

        result
    }

    /// Explicitly profiles a closure operation and captures a structured telemetry payload 
    /// without printing immediately (ideal for programmatic metric tracking).
    pub fn profile_block_structured<F, T>(label: &str, byte_count: usize, mut operation: F) -> (T, TransitionMetrics)
    where
        F: FnMut() -> T,
    {
        let start = Instant::now();
        let result = operation(); // Execute the actual streaming logic
        let duration = start.elapsed();

        // Calculate performance scaling numbers safely
        let megabytes = byte_count as f64 / (1024.0 * 1024.0);
        let seconds = duration.as_secs_f64();
        
        let throughput = if seconds > 0.0 {
            megabytes / seconds
        } else {
            0.0
        };

        let metrics = TransitionMetrics {
            subsystem_label: label.to_string(),
            bytes_processed: byte_count,
            execution_time: duration,
            throughput_mb_per_sec: throughput,
        };

        (result, metrics)
    }

    /// Pretty-prints telemetry feedback to stdout for developer logging
    pub fn log_metrics(metrics: &TransitionMetrics) {
        println!(
            "[METRIC] Subsystem: {:<25} | Size: {:>7.4} MB | Time: {:>9?} | Throughput: {:>8.2} MB/s",
            metrics.subsystem_label,
            metrics.bytes_processed as f64 / (1024.0 * 1024.0),
            metrics.execution_time,
            metrics.throughput_mb_per_sec
        );
    }
}
