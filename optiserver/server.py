from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List
import optimization_core

app = FastAPI(title="Computational Optimization Server", version="1.0.0")

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