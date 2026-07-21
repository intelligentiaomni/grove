# Grove

## Forecasts with Machine-Verifiable Provenance

Grove is a provenance-aware AI computation engine that transforms heterogeneous datasets into traceable forecasts. Every transformation, prediction, and optimization step is represented as a lineage graph and recorded in an append-only ledger, allowing developers and researchers to inspect how results were produced instead of treating AI outputs as opaque black boxes.

In one sentence: Grove makes AI forecasts reproducible by treating provenance as a first-class part of computation rather than an afterthought.

## Abstract

Suppose a team receives telemetry from hundreds of sensors, research papers, and incident reports. Existing workflows summarize those inputs. Grove instead builds a provenance graph, computes forecasts, and records every inference in an immutable lineage ledger. The result is not merely a prediction, but a forecast whose inputs, transformations, intermediate computations, confidence, and recommendation path can be queried by humans and machines.

The technical contribution is the combination of data lineage, forecasting, and reproducibility in a single execution engine. Many systems can produce predictions; far fewer can answer: exactly which inputs, transformations, and intermediate computations produced this forecast?

## Problem

AI systems are increasingly used in research, engineering, operations, and regulated domains, but their outputs are often detached from the computation path that produced them. A dashboard may show a forecast, yet the user cannot easily inspect:

- which raw records were consumed
- which parser and feature extraction steps touched the data
- which intermediate graph nodes influenced the result
- which confidence estimate was computed
- which optimization or recommendation step followed
- which immutable evidence exists for later audit

This opacity slows debugging, weakens trust, and makes scientific or operational replay difficult.

## Solution

Grove treats provenance as part of the computation itself. Each execution moves through a small, inspectable pipeline:

```text
CSV input
  -> Parser
  -> Knowledge graph
  -> Forecast
  -> Optimization
  -> Recommendation
  -> Ledger entry
  -> REST API and UI
```

Every edge in that pipeline carries provenance metadata: input hash, output hash, operation name, timestamp, evidence, and reasoning. The MVP exposes this through three live API surfaces:

```text
GET /forecast
GET /lineage
GET /graph
```

The point is intentionally narrow: a working vertical slice that demonstrates forecasts with machine-verifiable provenance.

## Architecture

The submission freezes the architecture into five layers:

```text
UI
  -> Gateway API
  -> Engine
      -> ingest/
      -> lineage/
      -> graph/
      -> forecaster/
      -> ledger/
  -> Storage
```

For the MVP, the gateway and engine are implemented as a compact Python standard-library service in `services/gateway_api.py`. The service parses telemetry CSV, computes a simple next-value forecast, builds a graph of the computation path, and appends every transition to `ledger/transitions.jsonl`.

The repository also contains deeper Rust research-engine modules under `engine/`. Those modules show the longer-term substrate direction: typed execution, graph lineage, forecasting, ingestion, kernel experimentation, and reproducible research workflows.

## Demo

Start the vertical slice:

```bash
docker compose up --build
```

Then open:

```text
http://localhost:5173
```

Run the API directly:

```bash
curl http://localhost:8080/forecast
curl http://localhost:8080/lineage
curl http://localhost:8080/graph
```

Ingest the sample telemetry file:

```bash
curl -X POST http://localhost:8080/ingest \
  -H "Content-Type: application/json" \
  -d @demo/payloads/telemetry_ingest.json
```

The ledger is written to:

```text
ledger/transitions.jsonl
```

## Demo Flow

The 3-4 minute demo is scripted in `demo/demo_script.md`.

The message of the demo is simple:

```text
Existing workflows summarize heterogeneous evidence.
Grove turns heterogeneous evidence into traceable forecasts.
Every screen reinforces the same capability:
forecasts with machine-verifiable provenance.
```

## Roadmap

- Add PDF ingestion with structured citation extraction.
- Add JSON incident-report ingestion beside telemetry CSV.
- Replace the MVP forecaster with pluggable model backends.
- Store graph and ledger state in SQLite/Postgres for concurrent execution.
- Add replay verification: recompute a forecast from ledger entries and compare hashes.
- Add signed ledger checkpoints for regulated environments.
- Extend the UI from graph inspection to replay, diffing, and confidence diagnostics.

## Aviation Analogy

Before formal aeronautics, humanity looked to the skies through lighter-than-air craft, myths, kites in ancient China, Leonardo da Vinci's conceptual sketches, and unpowered gliders such as those attributed to Abbas Ibn Firnas in the 9th century. Grove treats AI reproducibility as a similar transition point: moving from impressive flight-like outputs toward the engineering discipline required to explain, replay, and certify how those outputs were produced.

## Submission Materials

- Project description: this README
- Demo script: `demo/demo_script.md`
- Sample ingest payload: `demo/payloads/telemetry_ingest.json`
- API service: `services/gateway_api.py`
- UI: `ui/`
- Ledger output: `ledger/transitions.jsonl`
