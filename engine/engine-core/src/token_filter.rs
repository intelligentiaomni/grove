use std::collections::HashMap;

/// Maximum number of states allowed in our static compile phase
const MAX_STATES: usize = 256;
const FAIL_STATE: usize = 0;
const ROOT_STATE: usize = 1;

/// Metadata tracking which keyword triggered a state completion
#[derive(Clone, Debug)]
pub enum KeywordType {
    Anchor(&'static str),
    Constraint(&'static str),
}

/// A highly compact, zero-allocation transition table node
struct StateNode {
    // Maps a raw byte to the next state ID
    transitions: [u16; 256],
    // State to fallback to when a match fails (built via BFS)
    fail: u16,
    // Output matches registered at this node
    outputs: Vec<KeywordType>,
}

impl Default for StateNode {
    fn default() -> Self {
        Self {
            transitions: [0; 256],
            fail: 0,
            outputs: Vec::new(),
        }
    }
}

pub struct SinglePassMatcher {
    state_table: Vec<StateNode>,
    threshold: f32,
}

impl SinglePassMatcher {
    /// Compiles the text pattern variables into a deterministic Aho-Corasick state tree.
    pub fn compile() -> Self {
        let mut state_table = vec![StateNode::default(), StateNode::default()]; // Index 0=Fail, 1=Root
        let mut next_state = 2;

        // Unified source parameters from your topic matching design
        let anchors = vec!["early stopping", "early-stopping", "stop training", "convergence criteria"];
        let constraints = vec!["edge", "memory", "hardware", "ram", "oom", "resource-constrained", "tinyml", "embedded"];

        // 1. Build the Forward Trie Structure
        let mut insert_phrase = |phrase: &'static str, kw_type: KeywordType| {
            let mut current = ROOT_STATE;
            for &b in phrase.as_bytes() {
                // Force ASCII lowercase mapping on the fly for case-insensitivity
                let lower_b = b.to_ascii_lowercase() as usize;
                if state_table[current].transitions[lower_b] == 0 {
                    if next_state >= MAX_STATES { panic!("Aho-Corasick state table overflow!"); }
                    state_table.push(StateNode::default());
                    state_table[current].transitions[lower_b] = next_state as u16;
                    current = next_state;
                    next_state += 1;
                } else {
                    current = state_table[current].transitions[lower_b] as usize;
                }
            }
            state_table[current].outputs.push(kw_type);
        };

        for a in anchors { insert_phrase(a, KeywordType::Anchor(a)); }
        for c in constraints { insert_phrase(c, KeywordType::Constraint(c)); }

        // 2. Compute Failure Links via BFS (Breadth-First Search)
        let mut queue = Vec::new();
        
        for b in 0..256 {
            let u = state_table[ROOT_STATE].transitions[b] as usize;
            if u != 0 {
                state_table[u].fail = ROOT_STATE as u16;
                queue.push(u);
            } else {
                state_table[ROOT_STATE].transitions[b] = ROOT_STATE as u16; // Loopback
            }
        }

        while !queue.is_empty() {
            let r = queue.remove(0);
            for b in 0..256 {
                let u = state_table[r].transitions[b] as usize;
                if u != 0 {
                    queue.push(u);
                    let mut v = state_table[r].fail as usize;
                    while state_table[v].transitions[b] == 0 && v != FAIL_STATE {
                        v = state_table[v].fail as usize;
                    }
                    let fail_state = if v == FAIL_STATE { ROOT_STATE } else { state_table[v].transitions[b] as usize };
                    state_table[u].fail = fail_state as u16;
                    
                    // Merge outputs from the failure node clone path to catch nested matches
                    let additional_outputs = state_table[fail_state].outputs.clone();
                    state_table[u].outputs.extend(additional_outputs);
                }
            }
        }

        Self { state_table, threshold: 1.5 }
    }

    /// Evaluates an unvalidated streaming byte slice directly from a network socket ring.
    /// Emits matched topic string tags if the validation score parameters pass.
    pub fn scan_raw_bytes(&self, raw_buffer: &[u8]) -> Option<Vec<String>> {
        let mut current_state = ROOT_STATE;
        let mut anchor_hit = false;
        let mut unique_matches = HashMap::new();

        // Single linear execution path ($O(N)$ streaming evaluation)
        for &b in raw_buffer {
            let lower_b = b.to_ascii_lowercase() as usize;
            
            while self.state_table[current_state].transitions[lower_b] == 0 && current_state != FAIL_STATE {
                current_state = self.state_table[current_state].fail as usize;
            }
            
            current_state = if current_state == FAIL_STATE { ROOT_STATE } else { self.state_table[current_state].transitions[lower_b] as usize };

            if !self.state_table[current_state].outputs.is_empty() {
                for output in &self.state_table[current_state].outputs {
                    match output {
                        KeywordType::Anchor(name) => {
                            anchor_hit = true;
                            unique_matches.insert(name.replace(" ", "-"), 1.0f32);
                        }
                        KeywordType::Constraint(name) => {
                            unique_matches.insert(name.to_string(), 0.5f32);
                        }
                    }
                }
            }
        }

        // Fast escape path: if no anchoring keywords match, terminate processing
        if !anchor_hit {
            return None;
        }

        // Calculate final weight distribution metrics
        let total_score: f32 = 1.0 + unique_matches.values().sum::<f32>();

        if total_score >= self.threshold {
            let mut topics: Vec<String> = unique_matches.into_keys().collect();
            topics.sort();
            Some(topics)
        } else {
            None
        }
    }
}