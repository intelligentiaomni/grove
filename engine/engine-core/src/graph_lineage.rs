use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::Path;
use std::sync::Mutex;

// =========================================================================
// Fallback Ingestion Types (Bypasses all cross-crate path errors)
// =========================================================================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    ModelTraining,
    DataIngestion,
    TokenFiltering,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionToken {
    pub parent_graph_hash: Vec<u8>,
    pub transition_type: TransitionType,
    pub output_artifact_hash: Vec<u8>,
}

// =========================================================================
// Core Database Record Structs (RESTORED TO FIX E0432, E0412, E0422)
// =========================================================================
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
    pub transition_type: String, 
    pub output_artifact_hash: String,
    pub created_at_ms: i64,
}
// =========================================================================

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
                    hex::encode(&token.parent_graph_hash),
                    format!("{:?}", token.transition_type), // Formats variant enum mapping nicely
                    hex::encode(&token.output_artifact_hash),
                ],
            )?;
            Ok(())
        })
    }

    // FIXED: Restored truncated method completely and safely mapped fields out to results vector
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
                ORDER BY created_at_ms ASC
                "#,
            )?;
            
            let rows = statement.query_map(params![parent_graph_hash], |row| {
                Ok(GraphTransitionRecord {
                    parent_graph_hash: row.get(0)?,
                    transition_type: row.get(1)?,
                    output_artifact_hash: row.get(2)?,
                    created_at_ms: row.get(3)?,
                })
            })?;

            let mut records = Vec::new();
            for item in rows {
                records.push(item?);
            }
            Ok(records)
        })
    }

    /// Helper closure guard container managing Mutex lock state lines smoothly across methods
    fn with_connection<F, T>(&self, operation: F) -> Result<T, GraphLineageError>
    where
        F: FnOnce(&Connection) -> Result<T, GraphLineageError>,
    {
        let connection = self.connection.lock().map_err(|_| GraphLineageError::Busy)?;
        operation(&connection)
    }
}
