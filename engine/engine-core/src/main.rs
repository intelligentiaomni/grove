pub mod insight_accelerator;

use insight_accelerator::InsightAccelerator;

fn main() {
    let engine = InsightAccelerator::new();

    // Representing an ingested string slice chunk extracted from FineWeb-Edu / FinePDFs
    let paper =
        "Evaluating an automated exception validation sequence to patch system error loops.";

    println!("--- EXECUTING COGNITIVE ACCELERATION ENGINE RUN --- \n");

    // Pass the production dataset string directly into the core math layer
    let discovery_metrics = engine.calculate_conceptual_collision(paper);

    println!("Engine Evaluation Response:");
    println!(
        " -> Collision Status: {}",
        discovery_metrics.spark_triggered
    );
    println!(
        " -> System Execution Actuator: {}",
        discovery_metrics.action_alert
    );
}
