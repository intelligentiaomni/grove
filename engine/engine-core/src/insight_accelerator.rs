use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchVector {
    pub id: String,
    pub parent_group: String,
    pub concepts: Vec<String>,
    pub evidence_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralIntersection {
    pub left_vector_id: String,
    pub right_vector_id: String,
    pub left_parent_group: String,
    pub right_parent_group: String,
    pub shared_concepts: Vec<String>,
    pub similarity_score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollisionResult {
    pub spark_triggered: bool,
    pub collision_target: Option<String>,
    pub confidence_score: f32,
    pub shared_concepts: Vec<String>,
    pub action_alert: String,
}

#[derive(Debug, Clone, Default)]
pub struct InsightAccelerator {
    vectors: Vec<ResearchVector>,
    parent_index: HashMap<String, Vec<usize>>,
}

impl InsightAccelerator {
    pub fn new() -> Self {
        Self::from_vectors(seed_research_vectors())
    }

    pub fn from_vectors(vectors: Vec<ResearchVector>) -> Self {
        let mut parent_index: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, vector) in vectors.iter().enumerate() {
            parent_index
                .entry(vector.parent_group.clone())
                .or_default()
                .push(idx);
        }

        Self {
            vectors,
            parent_index,
        }
    }

    pub fn vectors(&self) -> &[ResearchVector] {
        &self.vectors
    }

    pub fn find_cross_parent_intersections(
        &self,
        inbound_concepts: &[String],
        minimum_shared_concepts: usize,
    ) -> Vec<StructuralIntersection> {
        let inbound_set = concept_set(inbound_concepts);
        if inbound_set.is_empty() {
            return Vec::new();
        }

        let mut candidates = Vec::new();
        for left_indices in self.parent_index.values() {
            for &left_idx in left_indices {
                let left = &self.vectors[left_idx];
                let left_set = concept_set(&left.concepts);
                let inbound_left_overlap = inbound_set
                    .intersection(&left_set)
                    .copied()
                    .collect::<HashSet<_>>();

                if inbound_left_overlap.is_empty() {
                    continue;
                }

                for right in self
                    .vectors
                    .iter()
                    .filter(|right| right.parent_group != left.parent_group)
                {
                    let right_set = concept_set(&right.concepts);
                    let shared = inbound_left_overlap
                        .intersection(&right_set)
                        .copied()
                        .collect::<HashSet<_>>();

                    if shared.len() < minimum_shared_concepts {
                        continue;
                    }

                    let mut shared_concepts = shared
                        .into_iter()
                        .map(str::to_string)
                        .collect::<Vec<String>>();
                    shared_concepts.sort();

                    let denominator = left_set.union(&right_set).count().max(1) as f32;
                    candidates.push(StructuralIntersection {
                        left_vector_id: left.id.clone(),
                        right_vector_id: right.id.clone(),
                        left_parent_group: left.parent_group.clone(),
                        right_parent_group: right.parent_group.clone(),
                        similarity_score: shared_concepts.len() as f32 / denominator,
                        shared_concepts,
                    });
                }
            }
        }

        candidates.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.shared_concepts.len().cmp(&a.shared_concepts.len()))
        });
        candidates
    }

    pub fn calculate_conceptual_collision(&self, inbound_research_text: &str) -> CollisionResult {
        let inbound_concepts = tokenize_concepts(inbound_research_text);
        let intersections = self.find_cross_parent_intersections(&inbound_concepts, 2);

        let Some(best) = intersections.first() else {
            return CollisionResult {
                spark_triggered: false,
                collision_target: None,
                confidence_score: 0.0,
                shared_concepts: Vec::new(),
                action_alert:
                    "NOMINAL REASONING: no distant parent-group structural bridge detected."
                        .to_string(),
            };
        };

        CollisionResult {
            spark_triggered: true,
            collision_target: Some(format!(
                "{} -> {}",
                best.left_parent_group, best.right_parent_group
            )),
            confidence_score: best.similarity_score,
            shared_concepts: best.shared_concepts.clone(),
            action_alert: format!(
                "AHA MOMENT: {} and {} share structural concepts {:?}.",
                best.left_vector_id, best.right_vector_id, best.shared_concepts
            ),
        }
    }
}

pub fn tokenize_concepts(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() > 2)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn concept_set(concepts: &[String]) -> HashSet<&str> {
    concepts.iter().map(String::as_str).collect()
}

fn seed_research_vectors() -> Vec<ResearchVector> {
    vec![
        ResearchVector {
            id: "fineweb_supply_chain_repair".to_string(),
            parent_group: "software_supply_chain".to_string(),
            concepts: words("dependency repair validation rollback automation"),
            evidence_hash: "seed:supply-chain".to_string(),
        },
        ResearchVector {
            id: "fineweb_factory_cell".to_string(),
            parent_group: "factory_automation".to_string(),
            concepts: words("sensor calibration validation isolation automation"),
            evidence_hash: "seed:factory".to_string(),
        },
        ResearchVector {
            id: "fineweb_energy_grid".to_string(),
            parent_group: "regional_energy_grid".to_string(),
            concepts: words("voltage load isolation rollback stabilization"),
            evidence_hash: "seed:energy".to_string(),
        },
        ResearchVector {
            id: "fineweb_biotech_assay".to_string(),
            parent_group: "biotech_lab".to_string(),
            concepts: words("assay calibration validation mutation repair"),
            evidence_hash: "seed:biotech".to_string(),
        },
    ]
}

fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::InsightAccelerator;

    #[test]
    fn detects_structural_intersections_across_distant_parent_groups() {
        let accelerator = InsightAccelerator::new();

        let result = accelerator.calculate_conceptual_collision(
            "A repair automation study proposes validation and rollback after calibration drift.",
        );

        assert!(result.spark_triggered);
        assert!(result.confidence_score > 0.0);
        assert!(result
            .shared_concepts
            .iter()
            .any(|term| term == "validation"));
    }
}
