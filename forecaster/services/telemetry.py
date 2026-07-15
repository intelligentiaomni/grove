from enum import Enum
from typing import Optional
from pydantic import BaseModel, Field


class TwinKind(str, Enum):
    SoftwareSupplyChain = "SoftwareSupplyChain"
    FactoryAutomation = "FactoryAutomation"
    RegionalEnergyGrid = "RegionalEnergyGrid"


class TelemetryDecision(str, Enum):
    Clean = "Clean"
    Imputed = "Imputed"
    Quarantine = "Quarantine"


class TelemetryPayload(BaseModel):
    twin_id: str
    metric: str
    value: float
    timestamp_utc: str


class TelemetryBounds(BaseModel):
    metric: str
    min: float
    max: float
    max_delta_per_ms: Optional[float] = None


class TelemetryValidationReport(BaseModel):
    decision: TelemetryDecision
    reason: str


class TwinState(BaseModel):
    resilience: float = Field(..., ge=0.0, le=1.0)
    throughput: float = Field(..., ge=0.0, le=1.0)
    risk: float = Field(..., ge=0.0, le=1.0)
