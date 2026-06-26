#![allow(dead_code)]

mod token_filter;
mod hf_ingest;
mod pipeline;
mod graph_lineage; 
pub mod insight_accelerator;


use std::process::Command;
use std::time::Duration;
use pipeline::PipelineOrchestrator;
use pipeline::m1_state::IngestionAction;
use insight_accelerator::InsightAccelerator;

/// Executes the external Python DAG visualizer to refresh graph dependencies.
pub fn trigger_dag_rebuild() {
    println!("\n--- Auto-Regenerating Lineage Graphs ---");
    
    let mut output = Command::new("python3")
        .arg("scripts/m4_visualizer.py")
        .output();
        
    if output.is_err() {
        output = Command::new("python")
            .arg("scripts/m4_visualizer.py")
            .output();
    }
        
    match output {
        Ok(res) => {
            if res.status.success() {
                println!("[DAG VISUALIZER] Refreshed graph dependencies successfully.");
            } else {
                let stderr = String::from_utf8_lossy(&res.stderr);
                eprintln!("[DAG VISUALIZER] Script exited with error: {}", stderr.trim());
            }
        }
        Err(e) => {
            eprintln!("[DAG VISUALIZER] Failed to find or invoke Python interpreter: {}", e);
        }
    }
}

fn main() {
    println!("=== Launching Starvation-Aware High-Performance Data Engine ===");
    
    // 1. Initialize orchestrator components and cognitive engines
    let mut orchestrator = PipelineOrchestrator::new();
    let engine = InsightAccelerator::new();
    
    // 2. Pre-fill action history with consecutive duplicate actions to test compaction thresholds
    let source_hash = "sha256_9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
    let duplicate_action = IngestionAction::BindExtractedNodes {
        source_id: source_hash.to_string(),
        nodes: vec!["edge".to_string(), "ram".to_string()],
    };
    orchestrator.action_history.push(duplicate_action.clone());
    orchestrator.action_history.push(duplicate_action.clone());

    println!("Initial Pre-Load History Log Count: {}", orchestrator.action_history.len());

    // 3. Manually establish our baseline dataset target inside the state machine
    orchestrator.state.datasets.push(hf_ingest::DatasetSource {
        id: source_hash.to_string(),
        repo: "HuggingFaceFW/fineweb-edu".to_string(),
        split: "train".to_string(),
        parquet_file: "data/001.parquet".to_string(),
        byte_range: "0-5000000".to_string(),
        extracted_topics: Vec::new(),
        provenance_node_bound: false,
    });

    // 4. Simulate a structured benchmark payload string block
    let trigger_patterns = b"This document discusses machine learning optimization. \
                             Implementing early stopping helps prevent model overfitting. \
                             However, when deploying onto edge hardware with strict RAM limits, \
                             standard patience values can trigger severe OOM faults. \
                             Evaluating an automated exception validation sequence to patch system error loops.";

    // Convert text payload segment safely for the cognitive math engine run
    if let Ok(paper_slice) = std::str::from_utf8(trigger_patterns) {
        println!("\n--- EXECUTING COGNITIVE ACCELERATION ENGINE RUN ---");
        let discovery_metrics = engine.calculate_conceptual_collision(paper_slice);
        println!("Engine Evaluation Response:");
        println!(" -> Collision Status: {}", discovery_metrics.spark_triggered);
        println!(" -> System Execution Actuator: {}", discovery_metrics.action_alert);
        
        if discovery_metrics.spark_triggered {
            println!(" [!] Conceptual collision detected. Elevating ingestion priority.");
        }
    }

    // 5. Ingest Active Stream Payload 
    println!("\n--- Processing Active Data Stream ---");
    let _ = hf_ingest::stream_range_request_loop(&mut orchestrator, trigger_patterns);
    
    std::thread::sleep(Duration::from_millis(15));

    // 6. Ingest Missing Stream 
    println!("\n--- Processing Delayed Stream Slice (Network Starvation) ---");
    let missing_payload_stream = b"";
    let _ = hf_ingest::stream_range_request_loop(&mut orchestrator, missing_payload_stream);

    // 7. Calculate aggregated overall system throughput metrics
    if let Some(last_run) = orchestrator.performance_log.last() {
        println!("\n--- Ingestion Benchmark Summary ---");
        println!("Total Bytes Scanned: {} bytes", last_run.bytes_processed);
        println!("Calculated Processing Speed: {:.2} MB/s", last_run.throughput_mb_per_sec);
    }

    // 8. Inspect final state guarantees to prove the node bindings completed
    println!("\n--- Final State Validation Check ---");
    if let Some(updated_ds) = orchestrator.state.datasets.iter().find(|d| d.id == source_hash) {
        println!("Provenance Node Bound: {}", updated_ds.provenance_node_bound);
        println!("Extracted Graph Topics: {:?}", updated_ds.extracted_topics);
    }

    // 9. Flush runtime memory out to physical transactional disk ledger
    println!("\n--- Freezing Active Ledger State ---");
    match orchestrator.persist_to_disk("ledger.json") {
        Ok(_) => {
            println!("Success. Ledger safely frozen to disk.");
            // 10. Auto-trigger the Python visualization engine map dependency generation
            trigger_dag_rebuild();
        },
        Err(e) => eprintln!("Storage failure: {}", e),
    }
}