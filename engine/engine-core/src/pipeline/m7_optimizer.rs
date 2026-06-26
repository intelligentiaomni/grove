use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use crate::pipeline::m1_state::IngestionAction;
use crate::hf_ingest::UnifiedLedger;

pub struct IdleOptimizer {
    /// Flag tracking whether the socket layer is starved for data
    pub is_network_throttled: AtomicBool,
    /// Threshold to prevent optimization routines from spinning too hot
    last_run_timestamp: std::sync::Mutex<Instant>,
}

impl IdleOptimizer {
    pub fn new() -> Self {
        Self {
            is_network_throttled: AtomicBool::new(false),
            last_run_timestamp: std::sync::Mutex::new(Instant::now()),
        }
    }

    /// Signals the optimizer that the network socket buffer is completely empty
    pub fn set_starvation_state(&self, throttled: bool) {
        self.is_network_throttled.store(throttled, Ordering::Relaxed);
    }

    /// Backwards compatibility wrapper for simple tracking passes
    pub fn handle_idle_cycle(&self) {
        if self.is_network_throttled.load(Ordering::Relaxed) {
            println!("[IDLE OPTIMIZER] Network starved. Running ledger state deduplication tasks...");
            std::thread::sleep(Duration::from_millis(50)); // Prevent hot-looping the CPU from older engine passes
        }
    }

    /// Evaluates if the current thread has stalled on the network.
    /// If starved, it executes quick maintenance tasks before checking the stream again.
    pub fn interleave_idle_maintenance(
        &self, 
        action_history: &mut Vec<IngestionAction>,
        ledger_state: &mut UnifiedLedger
    ) -> bool {
        // Safe check if we are starved for range request bytes
        if !self.is_network_throttled.load(Ordering::Relaxed) {
            return false; // Network is healthy; immediately return to processing bytes
        }

        let mut last_run = self.last_run_timestamp.lock().unwrap();
        if last_run.elapsed() < Duration::from_millis(10) {
            // Throttle maintenance cycles to avoid hammering the CPU cache lines
            std::thread::sleep(Duration::from_micros(250));
            return false;
        }
        *last_run = Instant::now();

        println!("[IDLE OPTIMIZER] Network socket starved. Executing inline maintenance tasks...");

        // TASK 1: In-place Transaction Log Compaction (Deduplicate trailing actions)
        let original_len = action_history.len();
        action_history.dedup_by(|a, b| {
            match (a, b) {
                // If the same nodes are being repeatedly bound to the same hash, compact them
                (IngestionAction::BindExtractedNodes { source_id: id_a, nodes: n_a }, 
                 IngestionAction::BindExtractedNodes { source_id: id_b, nodes: n_b }) => id_a == id_b && n_a == n_b,
                _ => false
            }
        });
        
        if action_history.len() < original_len {
            println!("[IDLE OPTIMIZER] Successfully compacted historical event logs.");
        }

        // TASK 2: Micro-sort in-memory dataset nodes for faster binary searching inside hf_ingest
        for ds in &mut ledger_state.sources.datasets {
            ds.extracted_topics.sort_unstable();
            ds.extracted_topics.dedup();
        }

        true
    }
}
