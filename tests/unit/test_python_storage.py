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
    def test_fetch_local_status_rejects_missing_database(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            db_path = Path(temp_dir) / "missing.db"
            with self.assertRaises(FileNotFoundError):
                fetch_local_status(db_path)
if __name__ == "__main__":
    unittest.main()
