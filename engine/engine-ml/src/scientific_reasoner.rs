use engine_core::{
    AutomatedExperimentJob, ConstraintOperator, ScientificHypothesisNode, SymbolicConstraint,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScientificPipelineRequest {
    pub hypothesis: ScientificHypothesisNode,
    pub experiment: AutomatedExperimentJob,
    pub observed_values: Vec<(String, f32)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScientificValidationReport {
    pub hypothesis_id: String,
    pub experiment_id: String,
    pub symbolic_constraints_passed: usize,
    pub symbolic_constraints_failed: usize,
    pub neural_alignment_score: f32,
    pub accepted: bool,
    pub next_action: String,
}

#[derive(Debug, Clone, Copy)]
pub struct HybridScientificReasoner {
    acceptance_threshold: f32,
}

impl HybridScientificReasoner {
    pub const fn new(acceptance_threshold: f32) -> Self {
        Self {
            acceptance_threshold,
        }
    }

    pub async fn validate(
        &self,
        request: &ScientificPipelineRequest,
    ) -> ScientificValidationReport {
        tokio::task::yield_now().await;

        let (passed, failed, symbolic_score) =
            validate_constraints(&request.hypothesis.constraints, &request.observed_values);
        let neural_alignment_score = neural_alignment_score(&request.hypothesis);
        let blended_score = (symbolic_score * 0.65) + (neural_alignment_score * 0.35);
        let accepted = failed == 0 && blended_score >= self.acceptance_threshold;
        let next_action = if accepted {
            "schedule automated experiment replication".to_string()
        } else if request.experiment.max_iterations == 0 {
            "halt pipeline and request protocol revision".to_string()
        } else {
            "collect additional evidence and revalidate constraints".to_string()
        };

        ScientificValidationReport {
            hypothesis_id: request.hypothesis.id.clone(),
            experiment_id: request.experiment.id.clone(),
            symbolic_constraints_passed: passed,
            symbolic_constraints_failed: failed,
            neural_alignment_score,
            accepted,
            next_action,
        }
    }
}

impl Default for HybridScientificReasoner {
    fn default() -> Self {
        Self::new(0.70)
    }
}

fn validate_constraints(
    constraints: &[SymbolicConstraint],
    observed_values: &[(String, f32)],
) -> (usize, usize, f32) {
    if constraints.is_empty() {
        return (0, 0, 1.0);
    }

    let mut passed = 0_usize;
    let mut failed = 0_usize;
    let mut weighted_total = 0.0_f32;
    let mut weighted_passed = 0.0_f32;

    for constraint in constraints {
        let weight = constraint.confidence_weight.max(0.0);
        weighted_total += weight;

        let Some((_, observed)) = observed_values
            .iter()
            .find(|(variable, _)| variable == &constraint.variable)
        else {
            failed += 1;
            continue;
        };

        if evaluate_constraint(*observed, constraint.operator, constraint.threshold) {
            passed += 1;
            weighted_passed += weight;
        } else {
            failed += 1;
        }
    }

    let score = if weighted_total > 0.0 {
        weighted_passed / weighted_total
    } else {
        passed as f32 / constraints.len() as f32
    };

    (passed, failed, score.clamp(0.0, 1.0))
}

fn evaluate_constraint(observed: f32, operator: ConstraintOperator, threshold: f32) -> bool {
    match operator {
        ConstraintOperator::Equal => (observed - threshold).abs() <= f32::EPSILON,
        ConstraintOperator::NotEqual => (observed - threshold).abs() > f32::EPSILON,
        ConstraintOperator::LessThan => observed < threshold,
        ConstraintOperator::LessThanOrEqual => observed <= threshold,
        ConstraintOperator::GreaterThan => observed > threshold,
        ConstraintOperator::GreaterThanOrEqual => observed >= threshold,
    }
}

fn neural_alignment_score(hypothesis: &ScientificHypothesisNode) -> f32 {
    if hypothesis.embedding_hint.is_empty() {
        return 0.5;
    }

    let energy = hypothesis
        .embedding_hint
        .iter()
        .map(|value| value.abs())
        .sum::<f32>();
    let normalized = energy / hypothesis.embedding_hint.len() as f32;

    normalized.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{HybridScientificReasoner, ScientificPipelineRequest};
    use engine_core::{
        AutomatedExperimentJob, ConstraintOperator, ScientificHypothesisNode, SymbolicConstraint,
    };

    #[tokio::test]
    async fn accepts_symbolically_valid_hypothesis_with_alignment() {
        let reasoner = HybridScientificReasoner::default();
        let request = ScientificPipelineRequest {
            hypothesis: ScientificHypothesisNode::new(
                "h1".to_string(),
                "Catalyst increases yield".to_string(),
                vec!["evidence-a".to_string()],
                vec![SymbolicConstraint::new(
                    "c1".to_string(),
                    "yield_delta".to_string(),
                    ConstraintOperator::GreaterThanOrEqual,
                    0.2,
                    1.0,
                )],
                vec![0.9, 0.8, 0.7],
            ),
            experiment: AutomatedExperimentJob::new(
                "x1".to_string(),
                "h1".to_string(),
                "run replicated catalyst assay".to_string(),
                vec!["input-a".to_string()],
                "yield increase".to_string(),
                3,
            ),
            observed_values: vec![("yield_delta".to_string(), 0.25)],
        };

        let report = reasoner.validate(&request).await;

        assert!(report.accepted);
        assert_eq!(report.symbolic_constraints_passed, 1);
        assert_eq!(report.symbolic_constraints_failed, 0);
    }

    #[tokio::test]
    async fn rejects_failed_symbolic_constraint() {
        let reasoner = HybridScientificReasoner::default();
        let request = ScientificPipelineRequest {
            hypothesis: ScientificHypothesisNode::new(
                "h2".to_string(),
                "Noise remains bounded".to_string(),
                Vec::new(),
                vec![SymbolicConstraint::new(
                    "c2".to_string(),
                    "noise".to_string(),
                    ConstraintOperator::LessThan,
                    0.1,
                    1.0,
                )],
                vec![1.0],
            ),
            experiment: AutomatedExperimentJob::new(
                "x2".to_string(),
                "h2".to_string(),
                "measure noise".to_string(),
                Vec::new(),
                "low noise".to_string(),
                1,
            ),
            observed_values: vec![("noise".to_string(), 0.5)],
        };

        let report = reasoner.validate(&request).await;

        assert!(!report.accepted);
        assert_eq!(report.symbolic_constraints_failed, 1);
    }
}
