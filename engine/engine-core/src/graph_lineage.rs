use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::Path;
use std::sync::Mutex;

use crate::{ExecutionToken, TransitionType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeRecord {
    pub node_hash: String,
    pub payload_hash: String,
    pub kind: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdgeRecord {
    pub parent_hash: String,
    pub child_hash: String,
    pub relation: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphTransitionRecord {
    pub parent_graph_hash: String,
    pub transition_type: TransitionType,
    pub output_artifact_hash: String,
    pub created_at_ms: i64,
}

#[derive(Debug)]
pub enum GraphLineageError {
    Busy,
    Sqlite(rusqlite::Error),
}

pub type LineageError = GraphLineageError;

impl Display for GraphLineageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Busy => f.write_str("graph lineage controller is busy"),
            Self::Sqlite(err) => write!(f, "sqlite lineage error: {err}"),
        }
    }
}

impl std::error::Error for GraphLineageError {}

impl From<rusqlite::Error> for GraphLineageError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct GraphLineageController {
    connection: Mutex<Connection>,
}

impl GraphLineageController {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GraphLineageError> {
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    pub fn in_memory() -> Result<Self, GraphLineageError> {
        let connection = Connection::open_in_memory()?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: Connection) -> Result<Self, GraphLineageError> {
        let controller = Self {
            connection: Mutex::new(connection),
        };
        controller.initialize()?;
        Ok(controller)
    }

    pub fn initialize(&self) -> Result<(), GraphLineageError> {
        self.with_connection(|connection| {
            connection.execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;

                CREATE TABLE IF NOT EXISTS graph_nodes (
                    node_hash TEXT PRIMARY KEY NOT NULL,
                    payload_hash TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    created_at_ms INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS graph_edges (
                    parent_hash TEXT NOT NULL,
                    child_hash TEXT NOT NULL,
                    relation TEXT NOT NULL,
                    created_at_ms INTEGER NOT NULL,
                    PRIMARY KEY(parent_hash, child_hash, relation),
                    FOREIGN KEY(parent_hash) REFERENCES graph_nodes(node_hash),
                    FOREIGN KEY(child_hash) REFERENCES graph_nodes(node_hash)
                );

                CREATE INDEX IF NOT EXISTS idx_graph_edges_parent
                    ON graph_edges(parent_hash);
                CREATE INDEX IF NOT EXISTS idx_graph_edges_child
                    ON graph_edges(child_hash);

                CREATE TABLE IF NOT EXISTS graph_transitions (
                    transition_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    parent_graph_hash TEXT NOT NULL,
                    transition_type TEXT NOT NULL,
                    output_artifact_hash TEXT NOT NULL,
                    created_at_ms INTEGER NOT NULL DEFAULT (
                        CAST(strftime('%s', 'now') AS INTEGER) * 1000
                    )
                );

                CREATE INDEX IF NOT EXISTS idx_graph_transitions_parent
                    ON graph_transitions(parent_graph_hash);
                "#,
            )?;
            Ok(())
        })
    }

    pub fn insert_node(&self, node: &GraphNodeRecord) -> Result<(), GraphLineageError> {
        self.with_connection(|connection| {
            connection.execute(
                r#"
                INSERT INTO graph_nodes (node_hash, payload_hash, kind, created_at_ms)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(node_hash) DO NOTHING
                "#,
                params![
                    node.node_hash,
                    node.payload_hash,
                    node.kind,
                    node.created_at_ms
                ],
            )?;
            Ok(())
        })
    }

    pub fn insert_edge(&self, edge: &GraphEdgeRecord) -> Result<(), GraphLineageError> {
        self.with_connection(|connection| {
            connection.execute(
                r#"
                INSERT INTO graph_edges (parent_hash, child_hash, relation, created_at_ms)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(parent_hash, child_hash, relation) DO NOTHING
                "#,
                params![
                    edge.parent_hash,
                    edge.child_hash,
                    edge.relation,
                    edge.created_at_ms
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_node(&self, node_hash: &str) -> Result<Option<GraphNodeRecord>, GraphLineageError> {
        self.with_connection(|connection| {
            connection
                .query_row(
                    r#"
                    SELECT node_hash, payload_hash, kind, created_at_ms
                    FROM graph_nodes
                    WHERE node_hash = ?1
                    "#,
                    params![node_hash],
                    |row| {
                        Ok(GraphNodeRecord {
                            node_hash: row.get(0)?,
                            payload_hash: row.get(1)?,
                            kind: row.get(2)?,
                            created_at_ms: row.get(3)?,
                        })
                    },
                )
                .optional()
                .map_err(GraphLineageError::from)
        })
    }

    pub fn children_of(
        &self,
        parent_hash: &str,
    ) -> Result<Vec<GraphEdgeRecord>, GraphLineageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                r#"
                SELECT parent_hash, child_hash, relation, created_at_ms
                FROM graph_edges
                WHERE parent_hash = ?1
                ORDER BY created_at_ms ASC
                "#,
            )?;
            let rows = statement.query_map(params![parent_hash], |row| {
                Ok(GraphEdgeRecord {
                    parent_hash: row.get(0)?,
                    child_hash: row.get(1)?,
                    relation: row.get(2)?,
                    created_at_ms: row.get(3)?,
                })
            })?;

            let mut edges = Vec::new();
            for edge in rows {
                edges.push(edge?);
            }
            Ok(edges)
        })
    }

    pub fn record_transition(&self, token: ExecutionToken) -> Result<(), LineageError> {
        self.with_connection(|connection| {
            connection.execute(
                r#"
                INSERT INTO graph_transitions (
                    parent_graph_hash,
                    transition_type,
                    output_artifact_hash
                )
                VALUES (?1, ?2, ?3)
                "#,
                params![
                    hex_encode(&token.parent_graph_hash),
                    transition_type_name(token.transition_type),
                    hex_encode(&token.output_artifact_hash),
                ],
            )?;
            Ok(())
        })
    }

    pub fn transitions_from(
        &self,
        parent_graph_hash: &str,
    ) -> Result<Vec<GraphTransitionRecord>, GraphLineageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                r#"
                SELECT parent_graph_hash, transition_type, output_artifact_hash, created_at_ms
                FROM graph_transitions
                WHERE parent_graph_hash = ?1
                ORDER BY transition_id ASC
                "#,
            )?;
            let rows = statement.query_map(params![parent_graph_hash], |row| {
                Ok(GraphTransitionRecord {
                    parent_graph_hash: row.get(0)?,
                    transition_type: parse_transition_type(row.get::<_, String>(1)?.as_str()),
                    output_artifact_hash: row.get(2)?,
                    created_at_ms: row.get(3)?,
                })
            })?;

            let mut transitions = Vec::new();
            for transition in rows {
                transitions.push(transition?);
            }
            Ok(transitions)
        })
    }

    fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, GraphLineageError>,
    ) -> Result<T, GraphLineageError> {
        let guard = self
            .connection
            .try_lock()
            .map_err(|_| GraphLineageError::Busy)?;
        operation(&guard)
    }
}

fn transition_type_name(transition_type: TransitionType) -> &'static str {
    match transition_type {
        TransitionType::OptimizationPass => "optimization_pass",
        TransitionType::KernelEvaluation => "kernel_evaluation",
        TransitionType::Compilation => "compilation",
        TransitionType::Execution => "execution",
    }
}

fn parse_transition_type(value: &str) -> TransitionType {
    match value {
        "optimization_pass" => TransitionType::OptimizationPass,
        "kernel_evaluation" => TransitionType::KernelEvaluation,
        "compilation" => TransitionType::Compilation,
        _ => TransitionType::Execution,
    }
}

fn hex_encode(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{GraphEdgeRecord, GraphLineageController, GraphLineageError, GraphNodeRecord};
    use crate::{ExecutionToken, TransitionType};

    #[test]
    fn stores_and_queries_graph_lineage_records() {
        let controller = GraphLineageController::in_memory().expect("db should initialize");
        let root = GraphNodeRecord {
            node_hash: "root-hash".to_string(),
            payload_hash: "payload-root".to_string(),
            kind: "root".to_string(),
            created_at_ms: 1,
        };
        let child = GraphNodeRecord {
            node_hash: "child-hash".to_string(),
            payload_hash: "payload-child".to_string(),
            kind: "derived".to_string(),
            created_at_ms: 2,
        };

        controller.insert_node(&root).expect("root insert");
        controller.insert_node(&child).expect("child insert");
        controller
            .insert_edge(&GraphEdgeRecord {
                parent_hash: root.node_hash.clone(),
                child_hash: child.node_hash.clone(),
                relation: "expands".to_string(),
                created_at_ms: 3,
            })
            .expect("edge insert");

        assert_eq!(
            controller.get_node("root-hash").expect("query root"),
            Some(root)
        );
        let children = controller.children_of("root-hash").expect("children query");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].child_hash, "child-hash");
    }

    #[test]
    fn busy_controller_returns_without_blocking() {
        let controller = GraphLineageController::in_memory().expect("db should initialize");
        let _guard = controller
            .connection
            .lock()
            .expect("test lock should acquire");

        let err = controller
            .get_node("anything")
            .expect_err("try_lock must fail while locked");

        assert!(matches!(err, GraphLineageError::Busy));
    }

    #[test]
    fn records_execution_tokens_as_append_only_transitions() {
        let controller = GraphLineageController::in_memory().expect("db should initialize");
        let token = ExecutionToken::new([1_u8; 32], TransitionType::OptimizationPass, [2_u8; 32]);

        controller
            .record_transition(token)
            .expect("transition insert");
        controller
            .record_transition(token)
            .expect("duplicate token is still a new ledger delta");

        let parent_hash = "0101010101010101010101010101010101010101010101010101010101010101";
        let transitions = controller
            .transitions_from(parent_hash)
            .expect("transition query");

        assert_eq!(transitions.len(), 2);
        assert_eq!(
            transitions[0].transition_type,
            TransitionType::OptimizationPass
        );
        assert_eq!(
            transitions[0].output_artifact_hash,
            "0202020202020202020202020202020202020202020202020202020202020202"
        );
    }
}
