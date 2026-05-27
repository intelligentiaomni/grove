use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicNode {
    pub what: String,
    pub who: Vec<String>,
    pub next_step: String,
}

impl TopicNode {
    pub fn new(what: String, who: Vec<String>, next_step: String) -> Self {
        Self {
            what,
            who,
            next_step,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrespondenceGraph {
    pub nodes: Vec<TopicNode>,
}

impl CorrespondenceGraph {
    pub fn new(nodes: Vec<TopicNode>) -> Self {
        Self { nodes }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
