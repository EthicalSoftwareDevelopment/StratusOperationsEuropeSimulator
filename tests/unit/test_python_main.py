import unittest

from app.main import collect_service_snapshot


class FakeClient:
    def health(self):
        return {"status": "ok"}

    def status(self):
        return {"event_count": 1}

    def topology(self):
        return {"local_node_id": "node-alpha"}

    def replication_state(self):
        return {"local_node_id": "node-alpha", "replication_peers": []}

    def security_baseline(self):
        return {"development_secret_in_use": True}


class PythonMainTests(unittest.TestCase):
    def test_collect_service_snapshot_aggregates_all_sections(self) -> None:
        snapshot = collect_service_snapshot(FakeClient())
        self.assertEqual(snapshot["health"]["status"], "ok")
        self.assertEqual(snapshot["status"]["event_count"], 1)
        self.assertEqual(snapshot["topology"]["local_node_id"], "node-alpha")
        self.assertEqual(snapshot["replication"]["replication_peers"], [])
        self.assertTrue(snapshot["security"]["development_secret_in_use"])


