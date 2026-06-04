use serde::{Deserialize, Serialize};

use crate::graph_ir::graph::ValidatedArchitectureIR;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub run_id: String,
    pub mode: ExecutionMode,
    pub budget: ExecutionBudget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    Local,
    Distributed,
    Simulation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionBudget {
    pub max_tokens: usize,
    pub max_steps: usize,
    pub max_latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub run_id: String,
    pub steps: Vec<ExecutionStep>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub node_id: String,
    pub action: ActionType,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Ingest,
    RouteInference,
    ScheduleTask,
    UpdateGraph,
    PersistArtifact,
    RenderVisualization,
}

pub trait ExecutableIR {
    fn exec(&self, ctx: ExecutionContext) -> ExecutionTrace;
}

impl ExecutableIR for ValidatedArchitectureIR {
    fn exec(&self, ctx: ExecutionContext) -> ExecutionTrace {
        let mut trace = ExecutionTrace {
            run_id: ctx.run_id.clone(),
            steps: vec![],
            success: true,
        };

        for node in &self.graph.nodes {
            trace.steps.push(ExecutionStep {
                node_id: node.id.clone(),
                action: ActionType::ScheduleTask,
                metadata: None,
            });
        }