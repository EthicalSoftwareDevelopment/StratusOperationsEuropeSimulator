# Windows Quickstart
## Rust service
```powershell
Set-Location C:\Dev\StratusOperationsEuropeSimulator\src\rust
cargo run
```
## Python CLI
```powershell
Set-Location C:\Dev\StratusOperationsEuropeSimulator\src\python
python -m app.main init-node --db ..\..\data\sql\node-bravo.db --node-id node-bravo --role replica
python -m app.main status --db ..\..\data\sql\node-bravo.db
python -m app.main probe-service --url http://127.0.0.1:8080
```
## Tests
```powershell
Set-Location C:\Dev\StratusOperationsEuropeSimulator\src\rust
cargo test
Set-Location C:\Dev\StratusOperationsEuropeSimulator
$env:PYTHONPATH = "C:\Dev\StratusOperationsEuropeSimulator\src\python"
python -m unittest discover -s tests\unit -p "test_*.py"
```
