use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolicConstraint {
    pub id: String,
    pub variable: String,
    pub operator: ConstraintOperator,
    pub threshold: f32,
    pub confidence_weight: f32,
}

impl SymbolicConstraint {
    pub fn new(
        id: String,
        variable: String,
        operator: ConstraintOperator,
        threshold: f32,
        confidence_weight: f32,
    ) -> Self {
        Self {
            id,
            variable,
            operator,
            threshold,
            confidence_weight,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScientificHypothesisNode {
    pub id: String,
    pub claim: String,
    pub evidence_hashes: Vec<String>,
    pub constraints: Vec<SymbolicConstraint>,
    pub embedding_hint: Vec<f32>,
}

impl ScientificHypothesisNode {
    pub fn new(
        id: String,
        claim: String,
        evidence_hashes: Vec<String>,
        constraints: Vec<SymbolicConstraint>,
        embedding_hint: Vec<f32>,
    ) -> Self {
        Self {
            id,
            claim,
            evidence_hashes,
            constraints,
            embedding_hint,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutomatedExperimentJob {
    pub id: String,
    pub hypothesis_id: String,
    pub protocol: String,
    pub input_artifact_hashes: Vec<String>,
    pub expected_observation: String,
    pub max_iterations: u32,
}

impl AutomatedExperimentJob {
    pub fn new(
        id: String,
        hypothesis_id: String,
        protocol: String,
        input_artifact_hashes: Vec<String>,
        expected_observation: String,
        max_iterations: u32,
    ) -> Self {
        Self {
            id,
            hypothesis_id,
            protocol,
            input_artifact_hashes,
            expected_observation,
            max_iterations,
        }
    }
}
