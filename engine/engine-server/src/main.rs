use anyhow::{bail, Context, Result};
use chrono::Utc;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Topo;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

/// Simple filesystem layout (under a root dir)
/// registry/transforms/*.json
/// states/*.json (content-addressed)
/// traces/traces.jsonl
const STORAGE_ROOT: &str = "kernel_store";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransformSpec {
    pub id: String,
    /// Exec command to run the transform. It is invoked with two positional args:
    /// <input_path> <output_path>
    /// Example: "python3 scripts/add.py" or "./bin/my_transform"
    pub exec_command: String,
    pub meta: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracePacket {
    pub trace_id: String,
    pub execution_id: String,
    pub transform_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub inputs_hash: String,
    pub outputs_hash: String,
    pub duration_ms: u128,
    pub resource_usage: serde_json::Value,
    pub error: Option<String>,
}

fn ensure_dirs() -> Result<()> {
    fs::create_dir_all(Path::new(STORAGE_ROOT).join("registry").join("transforms"))?;
    fs::create_dir_all(Path::new(STORAGE_ROOT).join("states"))?;
    fs::create_dir_all(Path::new(STORAGE_ROOT).join("traces"))?;
    Ok(())
}

/// Kernel Syscalls (minimal)
pub struct Kernel {}

impl Kernel {
    pub fn new() -> Result<Self> {
        ensure_dirs()?;
        Ok(Self {})
    }

    /// Register a transform in the registry (persist JSON)
    pub fn create_transform(&self, spec: &TransformSpec) -> Result<String> {
        let id = if spec.id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            spec.id.clone()
        };
        let path = Path::new(STORAGE_ROOT)
            .join("registry")
            .join("transforms")
            .join(format!("{}.json", id));
        let mut spec_with_id = spec.clone();
        spec_with_id.id = id.clone();
        let json = serde_json::to_vec_pretty(&spec_with_id)?;
        fs::write(path, json)?;
        Ok(id)
    }

    /// Persist state (content-addressed by SHA256 of payload)
    pub fn persist_state(&self, payload: &serde_json::Value) -> Result<String> {
        let bytes = serde_json::to_vec(payload)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hex::encode(hasher.finalize());
        let filename = format!("{}.json", hash);
        let path = Path::new(STORAGE_ROOT).join("states").join(&filename);
        if !path.exists() {
            fs::write(&path, &bytes)?;
        }
        Ok(hash)
    }

    /// Load state by hash
    pub fn load_state(&self, hash: &str) -> Result<serde_json::Value> {
        let path = Path::new(STORAGE_ROOT).join("states").join(format!("{}.json", hash));
        let b = fs::read(&path)
            .with_context(|| format!("state {} not found at {}", hash, path.display()))?;
        let v = serde_json::from_slice(&b)?;
        Ok(v)
    }

    /// Emit a low-level trace (append to traces.jsonl)
    pub fn emit_trace(&self, trace: &TracePacket) -> Result<()> {
        let path = Path::new(STORAGE_ROOT).join("traces").join("traces.jsonl");
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let json = serde_json::to_string(trace)?;
        writeln!(f, "{}", json)?;
        Ok(())
    }

    /// Naive execute_graph: takes a DAG defined by Node IDs and edges, executes transforms
    /// Assumption: each node has a single transform id and consumes outputs of predecessors as input merged as JSON.
    pub fn execute_graph(
        &self,
        graph_spec: GraphSpec,
        input_state_hash: &str,
    ) -> Result<String> {
        // load input state
        let mut state_inputs: HashMap<String, String> = HashMap::new();
        // map node name -> output state hash
        let mut outputs: HashMap<String, String> = HashMap::new();

        // Build petgraph graph
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        for node in &graph_spec.nodes {
            let idx = graph.add_node(node.name.clone());
            node_map.insert(node.name.clone(), idx);
        }
        for edge in &graph_spec.edges {
            let a = node_map
                .get(&edge.from)
                .context("edge from unknown node")?;
            let b = node_map.get(&edge.to).context("edge to unknown node")?;
            graph.add_edge(*a, *b, ());
        }

        // topological order
        let mut topo = Topo::new(&graph);
        let exec_id = Uuid::new_v4().to_string();

        // store the original input under a pseudo node "__input"
        let root_input_hash = input_state_hash.to_string();
        state_inputs.insert("__input".into(), root_input_hash);

        while let Some(nx) = topo.next(&graph) {
            let node_name = &graph[nx];
            // gather predecessors outputs
            let preds: Vec<_> = graph
                .neighbors_directed(nx, petgraph::Direction::Incoming)
                .map(|nidx| graph[nidx].clone())
                .collect();

            // merge predecessor outputs into one JSON (simple array or single item)
            let mut merged = Vec::new();
            if preds.is_empty() {
                // use root input
                let root = self.load_state(&root_input_hash)?;
                merged.push(root);
            } else {
                for p in preds {
                    if let Some(h) = outputs.get(&p) {
                        let st = self.load_state(h)?;
                        merged.push(st);
                    } else {
                        bail!("Missing output from predecessor {}", p);
                    }
                }
            }
            let merged_json = serde_json::Value::Array(merged);
            let input_hash = self.persist_state(&merged_json)?;

            // run transform for this node
            let node_spec = graph_spec
                .nodes
                .iter()
                .find(|n| &n.name == node_name)
                .context("node spec missing")?;
            let transform = self.load_transform(&node_spec.transform_id)?;
            let (output_hash, trace) = self.run_transform_with_io(&transform, &input_hash, &exec_id)?;

            // write trace
            self.emit_trace(&trace)?;
            outputs.insert(node_name.clone(), output_hash);
        }

        // final outputs: gather outputs of nodes marked as sinks
        let mut final_outputs = HashMap::new();
        for sink in &graph_spec.sinks {
            if let Some(h) = outputs.get(sink) {
                final_outputs.insert(sink.clone(), h.clone());
            }
        }

        // store a small execution summary state
        let summary = serde_json::json!({
            "execution_id": exec_id,
            "final_outputs": final_outputs,
            "timestamp": Utc::now()
        });
        let summary_hash = self.persist_state(&summary)?;
        Ok(summary_hash)
    }

    fn load_transform(&self, id: &str) -> Result<TransformSpec> {
        let path = Path::new(STORAGE_ROOT)
            .join("registry")
            .join("transforms")
            .join(format!("{}.json", id));
        let b = fs::read(&path)
            .with_context(|| format!("transform {} not found at {}", id, path.display()))?;
        let spec: TransformSpec = serde_json::from_slice(&b)?;
        Ok(spec)
    }

    /// Run the transform's exec_command by writing input to a temp file and calling:
    /// <exec_command> <input_path> <output_path>
    ///
    /// Returns (output_state_hash, TracePacket)
    fn run_transform_with_io(
        &self,
        transform: &TransformSpec,
        input_hash: &str,
        execution_id: &str,
    ) -> Result<(String, TracePacket)> {
        let input_val = self.load_state(input_hash)?;
        // write input temp file
        let input_file = Path::new(STORAGE_ROOT)
            .join("tmp")
            .join(format!("input-{}.json", Uuid::new_v4()));
        fs::create_dir_all(input_file.parent().unwrap())?;
        fs::write(&input_file, serde_json::to_vec(&input_val)?)?;

        let output_file = Path::new(STORAGE_ROOT)
            .join("tmp")
            .join(format!("output-{}.json", Uuid::new_v4()));

        // split exec_command into program + args (naive)
        let parts: Vec<&str> = transform.exec_command.split_whitespace().collect();
        if parts.is_empty() {
            bail!("empty exec_command for transform {}", transform.id);
        }
        let prog = parts[0];
        let args: Vec<&str> = parts[1..].to_vec();

        // Build command with input and output file args appended
        let mut cmd = Command::new(prog);
        for a in args {
            cmd.arg(a);
        }
        cmd.arg(input_file.as_os_str());
        cmd.arg(output_file.as_os_str());

        let t0 = std::time::Instant::now();
        let res = cmd.output()?;
        let duration = t0.elapsed().as_millis();

        if !res.status.success() {
            // capture stderr for trace error
            let err = String::from_utf8_lossy(&res.stderr).to_string();
            let trace = TracePacket {
                trace_id: Uuid::new_v4().to_string(),
                execution_id: execution_id.to_string(),
                transform_id: transform.id.clone(),
                timestamp: Utc::now(),
                inputs_hash: input_hash.to_string(),
                outputs_hash: "".to_string(),
                duration_ms: duration,
                resource_usage: serde_json::json!({ "exit_code": res.status.code() }),
                error: Some(err),
            };
            return Ok(("".to_string(), trace));
        }

        // read output file
        let output_bytes = fs::read(&output_file)
            .with_context(|| format!("expected transform to write output at {}", output_file.display()))?;
        let output_val: serde_json::Value = serde_json::from_slice(&output_bytes)?;
        let output_hash = self.persist_state(&output_val)?;

        let trace = TracePacket {
            trace_id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            transform_id: transform.id.clone(),
            timestamp: Utc::now(),
            inputs_hash: input_hash.to_string(),
            outputs_hash: output_hash.clone(),
            duration_ms: duration,
            resource_usage: serde_json::json!({ "output_bytes": output_bytes.len() }),
            error: None,
        };

        // cleanup temp files (optional)
        let _ = fs::remove_file(&input_file);
        let _ = fs::remove_file(&output_file);

        Ok((output_hash, trace))
    }
}

/// ---- Simple graph spec types ----
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphNode {
    pub name: String,
    pub transform_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphSpec {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    /// Optional: names of nodes whose outputs are final sinks
    pub sinks: Vec<String>,
}

/// ---- Demo CLI / example usage ----
fn main() -> Result<()> {
    // quick bootstrap
    let kernel = Kernel::new()?;

    // Example: create a tiny transform that copies input to output (using "cat" for demo).
    // Note: on Windows, replace "cat" with "type" or create a small script.
    let transform_spec = TransformSpec {
        id: "identity".into(),
        exec_command: "sh -c 'cat'".into(), // this runs `sh -c cat <in> <out>` -- works on unix shells
        meta: serde_json::json!({"desc":"identity demo (cat)"}),
    };
    let transform_id = kernel.create_transform(&transform_spec)?;
    println!("Created transform id = {}", transform_id);

    // Prepare an input state
    let input = serde_json::json!({"numbers":[1,2,3], "message":"hello"});
    let input_hash = kernel.persist_state(&input)?;
    println!("Persisted input state hash = {}", input_hash);

    // Graph: single node that takes the input and produces output
    let node = GraphNode {
        name: "node1".into(),
        transform_id: transform_id.clone(),
    };
    let graph = GraphSpec {
        nodes: vec![node],
        edges: vec![],
        sinks: vec!["node1".into()],
    };

    let exec_summary_hash = kernel.execute_graph(graph, &input_hash)?;
    println!("Execution summary stored at hash = {}", exec_summary_hash);

    // Print summary for convenience
    let summary = kernel.load_state(&exec_summary_hash)?;
    println!("Execution summary: {}", serde_json::to_string_pretty(&summary)?);

    Ok(())
}
