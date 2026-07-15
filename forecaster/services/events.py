from __future__ import annotations

import json
from typing import Any

import nats


class EventBus:
    """NATS event bus adapter."""

    def __init__(self, url: str) -> None:
        self._url = url
        self._nc = None

    async def connect(self) -> None:
        self._nc = await nats.connect(self._url)

    async def publish(self, subject: str, payload: dict[str, Any]) -> None:
        if self._nc is None:
            raise RuntimeError("Event bus is not connected.")

        await self._nc.publish(
            subject,
            json.dumps(payload).encode("utf-8"),
        )

    async def subscribe(self, subject: str, handler) -> None:
        if self._nc is None:
            raise RuntimeError("Event bus is not connected.")

        async def callback(msg):
            data = json.loads(msg.data.decode("utf-8"))
            await handler(data)

        await self._nc.subscribe(subject, cb=callback)

    async def close(self) -> None:
        if self._nc is not None:
            await self._nc.drain()