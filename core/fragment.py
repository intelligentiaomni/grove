from dataclasses import dataclass, field
from typing import List
import uuid
from datetime import datetime

@dataclass
class Fragment:
    id: str
    content: str
    source: str
    timestamp: str
    confidence: float = 0.0
    fitness: float = 0.0
    parents: List[str] = field(default_factory=list)
    tags: List[str] = field(default_factory=list)

    @staticmethod
    def create(content: str, source: str, confidence: float = 0.0):
        return Fragment(
            id=str(uuid.uuid4()),
            content=content,
            source=source,
            timestamp=datetime.utcnow().isoformat(),
            confidence=confidence,
        )
