from __future__ import annotations

import csv
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import urlparse


ROOT = Path(os.environ.get("GROVE_ROOT", Path(__file__).resolve().parents[1]))
LEDGER_DIR = Path(os.environ.get("GROVE_LEDGER_DIR", ROOT / "ledger"))
LEDGER_PATH = LEDGER_DIR / "transitions.jsonl"
DEMO_PAYLOAD = ROOT / "demo" / "payloads" / "telemetry_ingest.json"


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def stable_hash(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def append_ledger(entry: dict[str, Any]) -> None:
    LEDGER_DIR.mkdir(parents=True, exist_ok=True)
    with LEDGER_PATH.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(entry, sort_keys=True) + "\n")


def read_ledger() -> list[dict[str, Any]]:
    if not LEDGER_PATH.exists():
        return []
    entries: list[dict[str, Any]] = []
    with LEDGER_PATH.open("r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if line:
                entries.append(json.loads(line))
    return entries


def parse_csv(content: str) -> list[dict[str, str]]:
    return list(csv.DictReader(content.splitlines()))


def numeric_values(records: list[dict[str, str]]) -> list[float]:
    values: list[float] = []
    for record in records:
        raw = record.get("value") or record.get("reading") or record.get("measurement")
        if raw is None:
            continue
        try:
            values.append(float(raw))
        except ValueError:
            continue
    return values


def forecast_next(values: list[float]) -> dict[str, Any]:
    if not values:
        return {
            "next_value": None,
            "confidence": 0.0,
            "reasoning_chain": ["No numeric telemetry values were present."],
        }

    if len(values) == 1:
        return {
            "next_value": values[0],
            "confidence": 0.45,
            "reasoning_chain": ["One telemetry point was present; forecast repeats the observed value."],
        }

    deltas = [b - a for a, b in zip(values, values[1:])]
    mean_delta = sum(deltas) / len(deltas)
    variance = sum((delta - mean_delta) ** 2 for delta in deltas) / len(deltas)
    volatility = math.sqrt(variance)
    next_value = values[-1] + mean_delta
    confidence = max(0.25, min(0.95, 1.0 - (volatility / (abs(next_value) + 1.0))))

    return {
        "next_value": round(next_value, 3),
        "confidence": round(confidence, 3),
        "reasoning_chain": [
            f"Parsed {len(values)} numeric telemetry values.",
            f"Computed mean step change of {mean_delta:.3f}.",
            f"Projected next value from latest reading {values[-1]:.3f}.",
            "Bound confidence by observed delta volatility.",
        ],
    }


def build_graph(run_id: str, source_name: str, forecast: dict[str, Any]) -> dict[str, Any]:
    nodes = [
        {"id": "input", "label": source_name, "kind": "source"},
        {"id": "parser", "label": "CSV parser", "kind": "ingest"},
        {"id": "graph", "label": "Knowledge graph", "kind": "lineage"},
        {"id": "forecast", "label": "Forecast", "kind": "prediction"},
        {"id": "optimization", "label": "Optimization", "kind": "decision"},
        {"id": "recommendation", "label": "Recommendation", "kind": "output"},
    ]
    edges = [
        {"from": "input", "to": "parser", "operation": "parse_csv", "run_id": run_id},
        {"from": "parser", "to": "graph", "operation": "bind_records", "run_id": run_id},
        {"from": "graph", "to": "forecast", "operation": "forecast_next_value", "run_id": run_id},
        {"from": "forecast", "to": "optimization", "operation": "rank_response", "run_id": run_id},
        {"from": "optimization", "to": "recommendation", "operation": "emit_recommendation", "run_id": run_id},
    ]
    return {"run_id": run_id, "nodes": nodes, "edges": edges, "forecast": forecast}


def ingest(payload: dict[str, Any]) -> dict[str, Any]:
    source_name = str(payload.get("name", "telemetry.csv"))
    source_type = str(payload.get("type", "csv")).lower()
    content = str(payload.get("content", ""))
    if source_type != "csv":
        raise ValueError("MVP ingestion accepts CSV payloads. JSON and PDF are documented roadmap inputs.")

    records = parse_csv(content)
    values = numeric_values(records)
    forecast = forecast_next(values)
    run_id = stable_hash({"source": source_name, "content": content, "at": utc_now()})[:24]
    graph = build_graph(run_id, source_name, forecast)

    transitions = [
        ("input", "parser", "parse_csv", {"records": len(records)}),
        ("parser", "graph", "bind_records", {"nodes": len(graph["nodes"]), "edges": len(graph["edges"])}),
        ("graph", "forecast", "forecast_next_value", forecast),
        ("forecast", "optimization", "rank_response", {"policy": "highest-confidence-action"}),
        ("optimization", "recommendation", "emit_recommendation", {"recommendation": "inspect provenance before acting"}),
    ]

    previous_hash = stable_hash({"source_name": source_name, "content": content})
    for index, (source, target, operation, evidence) in enumerate(transitions, start=1):
        output_hash = stable_hash({"run_id": run_id, "index": index, "operation": operation, "evidence": evidence})
        append_ledger(
            {
                "id": f"{run_id}:{index}",
                "timestamp": utc_now(),
                "run_id": run_id,
                "from": source,
                "to": target,
                "operation": operation,
                "input_hash": previous_hash,
                "output_hash": output_hash,
                "evidence": evidence,
                "reasoning": f"{operation} produced {target} from {source}.",
            }
        )
        previous_hash = output_hash

    return {"run_id": run_id, "forecast": forecast, "graph": graph, "ledger_path": str(LEDGER_PATH)}


def latest_run() -> dict[str, Any]:
    entries = read_ledger()
    if not entries and DEMO_PAYLOAD.exists():
        with DEMO_PAYLOAD.open("r", encoding="utf-8") as handle:
            return ingest(json.load(handle))

    run_id = entries[-1]["run_id"] if entries else None
    run_entries = [entry for entry in entries if entry.get("run_id") == run_id]
    forecast_entry = next((entry for entry in run_entries if entry.get("operation") == "forecast_next_value"), None)
    forecast = forecast_entry["evidence"] if forecast_entry else {"next_value": None, "confidence": 0.0, "reasoning_chain": []}
    graph = build_graph(run_id or "empty", "latest telemetry", forecast)
    return {"run_id": run_id, "forecast": forecast, "graph": graph, "ledger_path": str(LEDGER_PATH)}


class GroveHandler(BaseHTTPRequestHandler):
    server_version = "GroveGateway/0.1"

    def end_headers(self) -> None:
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        super().end_headers()

    def send_json(self, status: int, body: Any) -> None:
        encoded = json.dumps(body, indent=2).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def do_OPTIONS(self) -> None:
        self.send_response(204)
        self.end_headers()

    def do_GET(self) -> None:
        path = urlparse(self.path).path.rstrip("/") or "/"
        try:
            if path == "/health":
                self.send_json(200, {"ok": True, "service": "grove-gateway"})
            elif path == "/forecast":
                self.send_json(200, latest_run()["forecast"])
            elif path == "/lineage":
                self.send_json(200, {"entries": read_ledger(), "ledger_path": str(LEDGER_PATH)})
            elif path == "/graph":
                self.send_json(200, latest_run()["graph"])
            elif path == "/demo/run":
                with DEMO_PAYLOAD.open("r", encoding="utf-8") as handle:
                    self.send_json(200, ingest(json.load(handle)))
            else:
                self.send_json(404, {"error": f"unknown endpoint: {path}"})
        except Exception as exc:
            self.send_json(500, {"error": str(exc)})

    def do_POST(self) -> None:
        path = urlparse(self.path).path.rstrip("/") or "/"
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length).decode("utf-8")
        try:
            payload = json.loads(body or "{}")
            if path == "/ingest":
                self.send_json(200, ingest(payload))
            else:
                self.send_json(404, {"error": f"unknown endpoint: {path}"})
        except Exception as exc:
            self.send_json(400, {"error": str(exc)})

    def log_message(self, format: str, *args: Any) -> None:
        print(f"[gateway] {self.address_string()} - {format % args}")


def main() -> None:
    port = int(os.environ.get("PORT", "8080"))
    server = ThreadingHTTPServer(("0.0.0.0", port), GroveHandler)
    print(f"Grove gateway listening on http://0.0.0.0:{port}")
    server.serve_forever()


if __name__ == "__main__":
    main()
