"""SQLite access layer: connection factory + migration runner.

SQLite is the ONLY source of truth (docs/erd.md v1.0). CI/CD safe: no ORM,
stdlib sqlite3 only. Migrations live in db/migrations/*.sql, applied in
filename order and recorded in schema_migrations.
"""

from __future__ import annotations

import datetime as dt
import logging
import os
import sqlite3
from pathlib import Path

DEFAULT_DB_PATH = os.environ.get("LOCAL_DB_PATH", "data/app.db")
MIGRATIONS_DIR = Path(__file__).resolve().parent / "db" / "migrations"

logger = logging.getLogger(__name__)


def get_db(db_path: str | Path | None = None) -> sqlite3.Connection:
    """Open a connection with WAL, row_factory and foreign_keys=ON."""
    path = Path(db_path or DEFAULT_DB_PATH)
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(path, timeout=30)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA foreign_keys = ON")
    return conn


def migrate(db_path: str | Path | None = None) -> int:
    """Apply pending migrations in filename order. Returns count applied."""
    path = Path(db_path or DEFAULT_DB_PATH)
    applied = 0
    files = sorted(MIGRATIONS_DIR.glob("*.sql"))
    if not files:
        return 0
    conn = get_db(path)
    try:
        current = {
            row[0] for row in conn.execute("SELECT version FROM schema_migrations").fetchall()
        } if _table_exists(conn, "schema_migrations") else set()
        for f in files:
            version = int(f.name.split("_", 1)[0])
            if version in current:
                continue
            sql = f.read_text(encoding="utf-8")
            logger.info("Applying migration %s", f.name)
            conn.executescript(sql)
            conn.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?, ?, ?)",
                (version, f.stem, dt.datetime.now(dt.timezone.utc).isoformat()),
            )
            conn.commit()
            applied += 1
        return applied
    finally:
        conn.close()


def _table_exists(conn: sqlite3.Connection, name: str) -> bool:
    return (
        conn.execute(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?", (name,)
        ).fetchone()
        is not None
    )
