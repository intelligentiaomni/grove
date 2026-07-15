from __future__ import annotations

import os

from sqlalchemy import create_engine
from sqlalchemy.engine import Engine
from sqlalchemy.orm import sessionmaker, Session


class Database:
    """PostgreSQL service adapter."""

    def __init__(self, url: str | None = None) -> None:
        self._url = url or os.environ["GROVE_DATABASE__URL"]

        self._engine: Engine = create_engine(
            self._url,
            pool_pre_ping=True,
            future=True,
        )

        self._session_factory = sessionmaker(
            bind=self._engine,
            autoflush=False,
            autocommit=False,
            future=True,
        )

    @property
    def engine(self) -> Engine:
        return self._engine

    def session(self) -> Session:
        return self._session_factory()

    def healthcheck(self) -> bool:
        try:
            with self._engine.connect() as conn:
                conn.exec_driver_sql("SELECT 1")
            return True
        except Exception:
            return False

    def dispose(self) -> None:
        self._engine.dispose()