use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct D3Node {
    pub id: String,
    pub label: String,
    pub group: String,
    pub weight: f32,
}

impl D3Node {
    pub fn new(id: String, label: String, group: String, weight: f32) -> Self {
        Self {
            id,
            label,
            group,
            weight,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct D3Link {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f32,
}

impl D3Link {
    pub fn new(source: String, target: String, relation: String, weight: f32) -> Self {
        Self {
            source,
            target,
            relation,
            weight,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VisualizationManifest {
    pub graph_id: String,
    pub generated_at_ms: i64,
    pub nodes: Vec<D3Node>,
    pub links: Vec<D3Link>,
}

impl VisualizationManifest {
    pub fn new(
        graph_id: String,
        generated_at_ms: i64,
        nodes: Vec<D3Node>,
        links: Vec<D3Link>,
    ) -> Self {
        Self {
            graph_id,
            generated_at_ms,
            nodes,
            links,
        }
    }
}
