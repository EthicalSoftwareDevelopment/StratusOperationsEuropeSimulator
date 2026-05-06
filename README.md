# StratusOperationsEuropeSimulator

Phase 1 foundation bootstrap for a hybrid Rust + Python simulation platform.

## What is implemented

- Rust foundation service in `src/rust`
- Python orchestration CLI in `src/python`
- Shared SQLite schema in `data/sql/001_init.sql`
- Windows quickstart in `docs/windows-quickstart.md`
- Docker scaffolding in `deployment/docker`

## Phase 1 scope currently covered

### 1.1 Programming Setup
- Rust application manifest and runnable service
- Python project manifest and runnable CLI

### 1.2 Data Storage
- SQLite-per-node storage model
- Shared schema for nodes, mission records, events, and replication state

### 1.3 Security Baseline
- Development-only event signing using SHA-256 and a shared secret
- Security baseline endpoint exposed by the Rust service
- Startup validation prevents unsafe default secrets on external interfaces

### 1.4 Networking Baseline
- Health, status, topology, replication-state, and event ingestion endpoints
- Bootstrap peer configuration via environment variables

## Quick start

### Run the Rust service

```powershell
Set-Location C:\Dev\StratusOperationsEuropeSimulator\src\rust
cargo run
```

### Initialize a SQLite node from Python

```powershell
Set-Location C:\Dev\StratusOperationsEuropeSimulator\src\python
python -m app.main init-node --db ..\..\data\sql\node-bravo.db --node-id node-bravo --role replica
python -m app.main status --db ..\..\data\sql\node-bravo.db
```

### Probe the running Rust service

```powershell
Set-Location C:\Dev\StratusOperationsEuropeSimulator\src\python
python -m app.main inspect-service --url http://127.0.0.1:8080
```

## Tests

```powershell
Set-Location C:\Dev\StratusOperationsEuropeSimulator\src\rust
cargo test

Set-Location C:\Dev\StratusOperationsEuropeSimulator
$env:PYTHONPATH = "C:\Dev\StratusOperationsEuropeSimulator\src\python"
python -m unittest discover -s tests\unit -p "test_*.py"
```

## Notes

- SQLite is intentionally file-based for Phase 1 and stored per simulated node.
- Replication is modeled in schema and configuration; active synchronization is a next iteration.
- The Rust service now exposes live replication-state inspection for peer visibility.
- Full Signal/WireGuard integration is not implemented yet; the current security work is a development baseline.
