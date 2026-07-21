# Grove Demo Script

Target length: 3 minutes 50 seconds.

## 0:00-0:20 - Problem

Show the README headline.

Narration:

> Teams receive telemetry, papers, and incident reports. Most AI workflows summarize the evidence. The hard question is not only what the model predicts, but exactly which inputs and intermediate computations produced that prediction.

Message on screen:

```text
Problem: AI forecasts are useful, but usually not replayable.
```

## 0:20-0:45 - Architecture

Show the frozen architecture:

```text
UI
  -> Gateway API
  -> Engine
      -> ingest
      -> lineage
      -> graph
      -> forecaster
      -> ledger
  -> Storage
```

Narration:

> Grove makes provenance part of the execution path. The engine ingests data, binds it into a graph, computes a forecast, and records every transition in an append-only ledger.

## 0:45-1:20 - Live Ingestion

Run:

```bash
curl -X POST http://localhost:8080/ingest \
  -H "Content-Type: application/json" \
  -d @demo/payloads/telemetry_ingest.json
```

Narration:

> This is the narrow MVP: CSV telemetry in, traceable forecast out. The same boundary can later accept JSON incident reports and PDFs, but the judging slice is deliberately small.

Show the returned `run_id`, `forecast`, and `ledger_path`.

## 1:20-2:00 - Forecast

Run:

```bash
curl http://localhost:8080/forecast
```

Narration:

> The forecast is not presented as an isolated answer. It includes the next value, confidence, and the reasoning chain used by the engine.

Message on screen:

```text
Next value + confidence + reasoning chain
```

## 2:00-2:40 - Lineage Graph

Open the UI at:

```text
http://localhost:5173
```

Show the graph:

```text
Node A
  -> Node B
  -> Forecast
  -> Optimization
  -> Recommendation
```

Narration:

> Every edge has provenance. The graph is intentionally simple: the point is not visual spectacle, but inspectability.

## 2:40-3:20 - Ledger

Run:

```bash
curl http://localhost:8080/lineage
```

Then open:

```text
ledger/transitions.jsonl
```

Narration:

> Every decision exists as a ledger transition. Each entry includes the operation, timestamp, input hash, output hash, evidence, and reasoning. This is what makes the forecast replayable and auditable.

## 3:20-3:50 - Future Vision

Narration:

> The future version ingests sensor streams, papers, and incident reports continuously. It computes forecasts, links them to source evidence, and lets researchers verify how each inference was produced. The core claim already works: forecasts with machine-verifiable provenance.

Final screen:

```text
Grove: provenance-aware AI computation for reproducible forecasts.
```
