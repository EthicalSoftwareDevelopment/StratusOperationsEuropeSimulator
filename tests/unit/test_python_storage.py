import sqlite3
import tempfile
import unittest
from pathlib import Path
from app.storage import fetch_local_status, initialize_node_database
class PythonStorageTests(unittest.TestCase):
    def test_initialize_node_database_creates_schema(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            db_path = Path(temp_dir) / "node-bravo.db"
            initialize_node_database(db_path, node_id="node-bravo", role="replica")
            status = fetch_local_status(db_path)
            self.assertEqual(status["node_count"], 1)
            self.assertEqual(status["mission_record_count"], 0)
            self.assertEqual(status["event_count"], 0)
            self.assertEqual(status["replication_peer_count"], 0)
            self.assertEqual(status["replication_state"], [])

    def test_fetch_local_status_includes_replication_state_rows(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            db_path = Path(temp_dir) / "node-bravo.db"
            initialize_node_database(db_path, node_id="node-bravo", role="replica")
            connection = sqlite3.connect(db_path)
            try:
                connection.execute(
                    "INSERT INTO replication_state (peer_node_id, last_replicated_event_id, last_sync_utc, status) VALUES (?, ?, ?, ?)",
                    ("node-alpha", 12, "2026-05-06T00:00:00Z", "synced"),
                )
                connection.commit()
            finally:
                connection.close()

            status = fetch_local_status(db_path)
            self.assertEqual(status["replication_peer_count"], 1)
            self.assertEqual(status["replication_state"][0]["peer_node_id"], "node-alpha")
    def test_fetch_local_status_rejects_missing_database(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            db_path = Path(temp_dir) / "missing.db"
            with self.assertRaises(FileNotFoundError):
                fetch_local_status(db_path)
if __name__ == "__main__":
    unittest.main()
