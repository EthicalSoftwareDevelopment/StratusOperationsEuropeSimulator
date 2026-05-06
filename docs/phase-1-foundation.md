# Phase 1 Foundation Implementation Notes
## Scope Covered
### 1.1 Programming Setup
- Rust workspace at `src/rust` with a runnable HTTP service.
- Python orchestration helpers at `src/python` with a simple CLI.
- Shared Windows-friendly quickstart in `docs/windows-quickstart.md`.
### 1.2 Data Storage (SQLite)
- Each simulated node owns its own SQLite file under `data/sql`.
- Shared schema in `data/sql/001_init.sql`.
- Rust and Python both initialize and query the same schema.
- Replication is tracked in `replication_state`; actual multi-node sync comes next.
### 1.3 Security Baseline
- Development-only event signing in Rust using SHA-256 hashes and a shared secret.
- Security posture exposed at `/api/security/baseline`.
- Proper key management, WireGuard, and Signal-style channels remain follow-on work.
### 1.4 Networking Baseline
- Rust service exposes health, topology, status, and event ingestion endpoints.
- Bootstrap peers are configured via `STRATUS_BOOTSTRAP_PEERS`.
- This iteration models node-to-node topology without implementing IGP/BGP yet.
## Immediate Next Steps
1. Add per-node keypairs and signature verification for events.
2. Implement replication from `events` into peer node databases.
3. Add Python workflows for analysts/operators on top of the Phase 1 CLI.
4. Introduce Docker Compose-driven multi-node integration tests.
