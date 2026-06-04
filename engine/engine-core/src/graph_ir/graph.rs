use serde::{Deserialize, Serialize};

use crate::graph_ir::edge::Edge;
use crate::graph_ir::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedArchitectureIR {
    pub graph: ArchitectureGraph,
    pub version: String,
}