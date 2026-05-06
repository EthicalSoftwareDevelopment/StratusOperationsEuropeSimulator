from __future__ import annotations
import sqlite3
from pathlib import Path
from typing import Any
from urllib.parse import urlparse
SCHEMA_PATH = Path(__file__).resolve().parents[3] / "data" / "sql" / "001_init.sql"
def load_schema() -> str:
    return SCHEMA_PATH.read_text(encoding="utf-8")
def initialize_node_database(db_path: str | Path, node_id: str, role: str = "replica", endpoint: str | None = None) -> Path:
    db_file = Path(db_path)
    db_file.parent.mkdir(parents=True, exist_ok=True)
    parsed = urlparse(endpoint or f"http://127.0.0.1/{node_id}")
    normalized_endpoint = endpoint or f"{parsed.scheme}://{parsed.netloc}{parsed.path}"
    with sqlite3.connect(db_file) as connection:
        connection.executescript(load_schema())
        connection.execute(
            """
            INSERT INTO nodes (node_id, role, status, endpoint, metadata_json, last_seen_utc)
            VALUES (?, ?, 'online', ?, ?, datetime('now'))
            ON CONFLICT(node_id) DO UPDATE SET
                role = excluded.role,
                status = excluded.status,
                endpoint = excluded.endpoint,
                metadata_json = excluded.metadata_json,
                last_seen_utc = excluded.last_seen_utc
            """,
            (node_id, role, normalized_endpoint, '{"created_by":"python-cli"}'),
        )
        connection.commit()
    return db_file
def fetch_local_status(db_path: str | Path) -> dict[str, Any]:
    db_file = Path(db_path)
    if not db_file.exists():
        raise FileNotFoundError(f"SQLite database not found: {db_file}")
    with sqlite3.connect(db_file) as connection:
        node_count = connection.execute("SELECT COUNT(*) FROM nodes").fetchone()[0]
        mission_record_count = connection.execute("SELECT COUNT(*) FROM mission_records").fetchone()[0]
        event_count = connection.execute("SELECT COUNT(*) FROM events").fetchone()[0]
        replication_peer_count = connection.execute("SELECT COUNT(*) FROM replication_state").fetchone()[0]
    return {
        "database": str(db_file),
        "node_count": node_count,
        "mission_record_count": mission_record_count,
        "event_count": event_count,
        "replication_peer_count": replication_peer_count,
    }
