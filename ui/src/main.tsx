import React, { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

type Forecast = {
  next_value: number | null;
  confidence: number;
  reasoning_chain: string[];
};

type GraphNode = {
  id: string;
  label: string;
  kind: string;
};

type GraphEdge = {
  from: string;
  to: string;
  operation: string;
  run_id: string;
};

type Graph = {
  run_id: string;
  nodes: GraphNode[];
  edges: GraphEdge[];
  forecast: Forecast;
};

type LedgerEntry = {
  id: string;
  timestamp: string;
  from: string;
  to: string;
  operation: string;
  input_hash: string;
  output_hash: string;
  reasoning: string;
};

const samplePayload = {
  name: "reactor-alpha-telemetry.csv",
  type: "csv",
  content:
    "timestamp,sensor,value\n2026-07-21T12:00:00Z,temp-A,68.2\n2026-07-21T12:01:00Z,temp-A,69.1\n2026-07-21T12:02:00Z,temp-A,70.4\n2026-07-21T12:03:00Z,temp-A,71.0\n2026-07-21T12:04:00Z,temp-A,72.3\n",
};

async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json() as Promise<T>;
}

function App() {
  const [graph, setGraph] = useState<Graph | null>(null);
  const [forecast, setForecast] = useState<Forecast | null>(null);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [status, setStatus] = useState("Ready");

  async function refresh() {
    const [nextForecast, nextGraph, lineage] = await Promise.all([
      getJson<Forecast>("/api/forecast"),
      getJson<Graph>("/api/graph"),
      getJson<{ entries: LedgerEntry[] }>("/api/lineage"),
    ]);
    setForecast(nextForecast);
    setGraph(nextGraph);
    setLedger(lineage.entries.slice(-8).reverse());
  }

  async function ingestSample() {
    setStatus("Ingesting sample telemetry");
    const response = await fetch("/api/ingest", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(samplePayload),
    });
    if (!response.ok) {
      setStatus("Ingestion failed");
      return;
    }
    await refresh();
    setStatus("Ledger updated");
  }

  useEffect(() => {
    refresh().catch(() => setStatus("Start the API on port 8080"));
  }, []);

  const confidence = useMemo(() => {
    if (!forecast) return "0%";
    return `${Math.round(forecast.confidence * 100)}%`;
  }, [forecast]);

  return (
    <main>
      <header className="topbar">
        <div>
          <p className="eyebrow">Grove</p>
          <h1>Forecasts with machine-verifiable provenance</h1>
        </div>
        <button onClick={ingestSample}>Ingest CSV</button>
      </header>

      <section className="summary">
        <div>
          <span>Next value</span>
          <strong>{forecast?.next_value ?? "pending"}</strong>
        </div>
        <div>
          <span>Confidence</span>
          <strong>{confidence}</strong>
        </div>
        <div>
          <span>Status</span>
          <strong>{status}</strong>
        </div>
      </section>

      <section className="workspace">
        <div className="graph" aria-label="Lineage graph">
          {(graph?.nodes ?? []).map((node, index) => (
            <React.Fragment key={node.id}>
              <div className={`node ${node.kind}`}>
                <span>{node.kind}</span>
                <strong>{node.label}</strong>
              </div>
              {index < (graph?.nodes.length ?? 0) - 1 && <div className="edge">provenance edge</div>}
            </React.Fragment>
          ))}
        </div>

        <aside className="panel">
          <h2>Reasoning chain</h2>
          <ol>
            {(forecast?.reasoning_chain ?? []).map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
        </aside>
      </section>

      <section className="ledger">
        <div className="sectionTitle">
          <h2>Latest ledger transitions</h2>
          <code>ledger/transitions.jsonl</code>
        </div>
        <div className="rows">
          {ledger.map((entry) => (
            <article key={entry.id}>
              <div>
                <strong>{entry.operation}</strong>
                <span>
                  {entry.from} {"->"} {entry.to}
                </span>
              </div>
              <code>{entry.output_hash.slice(0, 22)}</code>
            </article>
          ))}
        </div>
      </section>
    </main>
  );
}

createRoot(document.getElementById("root")!).render(<App />);
