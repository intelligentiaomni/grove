import json
import threading
from pathlib import Path
from time import time
from typing import List, Optional

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import optimization_core

app = FastAPI(title="Computational Optimization Server", version="1.0.0")
LINEAGE_LOG_PATH = Path(__file__).with_name("lineage_append_log.jsonl")
LINEAGE_LOCK = threading.Lock()
LINEAGE_PACKETS = []

# --- STEP 1: Serialize Data Contracts ---
class NodeSchema(BaseModel):
    id: str
    balance: float  # Positive = Injects to system, Negative = Consumes
class EdgeSchema(BaseModel):
    source: str
    target: str
    resistance: float  # Spatial network friction factor
class NetworkRequestPayload(BaseModel):
    system_id: str
    nodes: List[NodeSchema]
    edges: List[EdgeSchema]


class LineageNodePacket(BaseModel):
    node_id: str
    parent_hash: Optional[str] = None
    payload_sha256: str
    token_count: int
    epistemic_score: float
    source_kind: str = "unknown"
    created_at_ms: Optional[int] = None


@app.on_event("startup")
async def load_lineage_log():
    if not LINEAGE_LOG_PATH.exists():
        return

    with LINEAGE_LOCK:
        LINEAGE_PACKETS.clear()
        for line in LINEAGE_LOG_PATH.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            try:
                LINEAGE_PACKETS.append(json.loads(line))
            except json.JSONDecodeError:
                continue


# --- STEP 2: Asynchronous REST Endpoint ---
@app.post("/api/v1/optimize")
async def optimize_network(payload: NetworkRequestPayload):
    try:
        # Convert incoming JSON data structure to native core dict structures
        nodes_dict = [n.model_dump() for n in payload.nodes]
        edges_dict = [e.model_dump() for e in payload.edges]
        
        # Hand off execution to the isolated mathematical library core
        calculation_result = optimization_core.run_network_optimization(nodes_dict, edges_dict)
        
        return {
            "status": "success",
            "system_id": payload.system_id,
            "metrics": {
                "total_system_loss": calculation_result["minimized_loss"]
            },
            "routing_matrix": calculation_result["matrix"]
        }
    except Exception as err:
        raise HTTPException(status_code=422, detail=f"Mathematical calculation failure: {str(err)}")


@app.post("/api/v1/lineage")
async def ingest_lineage_packet(packet: LineageNodePacket):
    try:
        record = packet.model_dump()
        record["received_at_ms"] = int(time() * 1000)
        priority = optimization_core.prioritize_lineage_packet(record)
        record["priority_score"] = priority["priority_score"]
        record["deep_processing"] = priority["deep_processing"]

        with LINEAGE_LOCK:
            LINEAGE_PACKETS.append(record)
            with LINEAGE_LOG_PATH.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps(record, sort_keys=True) + "\n")

        return {
            "status": "accepted",
            "node_id": record["node_id"],
            "priority": priority,
        }
    except Exception as err:
        raise HTTPException(status_code=422, detail=f"Lineage packet rejection: {str(err)}")


@app.get("/api/v1/lineage/telemetry")
async def lineage_telemetry():
    with LINEAGE_LOCK:
        packets = list(LINEAGE_PACKETS)

    prioritized = optimization_core.prioritize_lineage_packets(packets)
    token_distribution = {
        "0-999": 0,
        "1000-9999": 0,
        "10000-24999": 0,
        "25000+": 0,
    }
    score_distribution = {
        "0.00-0.25": 0,
        "0.25-0.50": 0,
        "0.50-0.75": 0,
        "0.75-1.00": 0,
    }

    for packet in packets:
        token_count = int(packet.get("token_count", 0) or 0)
        score = float(packet.get("epistemic_score", 0.0) or 0.0)

        if token_count < 1_000:
            token_distribution["0-999"] += 1
        elif token_count < 10_000:
            token_distribution["1000-9999"] += 1
        elif token_count <= 24_999:
            token_distribution["10000-24999"] += 1
        else:
            token_distribution["25000+"] += 1

        if score < 0.25:
            score_distribution["0.00-0.25"] += 1
        elif score < 0.50:
            score_distribution["0.25-0.50"] += 1
        elif score < 0.75:
            score_distribution["0.50-0.75"] += 1
        else:
            score_distribution["0.75-1.00"] += 1

    return {
        "total_packets": len(packets),
        "deep_processing_candidates": sum(1 for item in prioritized if item["deep_processing"]),
        "token_distribution": token_distribution,
        "score_distribution": score_distribution,
        "recent_packets": packets[-25:],
        "priority_queue": prioritized[:25],
    }
