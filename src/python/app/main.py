from __future__ import annotations
import argparse
import json
from pathlib import Path
from typing import Any
from app.service_client import RustServiceClient
from app.storage import fetch_local_status, initialize_node_database
def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Stratus Phase 1 foundation CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)
    init_parser = subparsers.add_parser("init-node", help="Initialize a SQLite node database")
    init_parser.add_argument("--db", required=True, help="Path to the SQLite database file")
    init_parser.add_argument("--node-id", required=True, help="Logical node identifier")
    init_parser.add_argument("--role", default="replica", help="Node role")
    init_parser.add_argument("--endpoint", help="Advertised endpoint for the node")
    status_parser = subparsers.add_parser("status", help="Read local SQLite status")
    status_parser.add_argument("--db", required=True, help="Path to the SQLite database file")
    probe_parser = subparsers.add_parser("probe-service", help="Query the Rust foundation service")
    probe_parser.add_argument("--url", required=True, help="Base URL for the Rust service")
    inspect_parser = subparsers.add_parser("inspect-service", help="Collect a complete Rust service snapshot")
    inspect_parser.add_argument("--url", required=True, help="Base URL for the Rust service")
    return parser


def collect_service_snapshot(client: Any) -> dict[str, object]:
    return {
        "health": client.health(),
        "status": client.status(),
        "topology": client.topology(),
        "replication": client.replication_state(),
        "security": client.security_baseline(),
    }


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    if args.command == "init-node":
        database_path = initialize_node_database(
            db_path=Path(args.db),
            node_id=args.node_id,
            role=args.role,
            endpoint=args.endpoint,
        )
        print(json.dumps({"initialized": str(database_path), "node_id": args.node_id}, indent=2))
        return 0
    if args.command == "status":
        print(json.dumps(fetch_local_status(args.db), indent=2))
        return 0
    if args.command in {"probe-service", "inspect-service"}:
        client = RustServiceClient(args.url)
        print(json.dumps(collect_service_snapshot(client), indent=2))
        return 0
    parser.error("Unsupported command")
    return 2
if __name__ == "__main__":
    raise SystemExit(main())
