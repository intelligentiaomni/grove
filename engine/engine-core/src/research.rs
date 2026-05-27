use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchEvent {
    pub thread_id: String,
    pub sequence: u64,
    pub source: String,
    pub payload: String,
}

impl ResearchEvent {
    pub fn new(thread_id: String, sequence: u64, source: String, payload: String) -> Self {
        Self {
            thread_id,
            sequence,
            source,
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerAction {
    Continue {
        thread_id: String,
        next_query: String,
    },
    Defer {
        thread_id: String,
        reason: String,
    },
    Fork {
        thread_id: String,
        child_thread_id: String,
        reason: String,
    },
    Complete {
        thread_id: String,
        summary: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchThread {
    pub id: String,
    pub title: String,
    pub events: Vec<ResearchEvent>,
    pub last_action: Option<SchedulerAction>,
}

impl ResearchThread {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            events: Vec::new(),
            last_action: None,
        }
    }
}
