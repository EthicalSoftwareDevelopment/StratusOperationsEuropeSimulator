from __future__ import annotations
import json
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen
class RustServiceClient:
    def __init__(self, base_url: str, timeout: float = 5.0) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
    def get_json(self, path: str) -> dict[str, Any] | list[dict[str, Any]]:
        request = Request(f"{self.base_url}{path}", headers={"Accept": "application/json"})
        try:
            with urlopen(request, timeout=self.timeout) as response:
                return json.loads(response.read().decode("utf-8"))
        except HTTPError as error:  # pragma: no cover - exercised by manual probing
            raise RuntimeError(f"HTTP {error.code}: {error.reason}") from error
        except URLError as error:
            raise RuntimeError(f"Connection failed: {error.reason}") from error
    def health(self) -> dict[str, Any]:
        return self.get_json("/health")
    def status(self) -> dict[str, Any]:
        return self.get_json("/api/status")
    def topology(self) -> dict[str, Any]:
        return self.get_json("/api/network/topology")

    def replication_state(self) -> dict[str, Any]:
        return self.get_json("/api/replication/state")

    def security_baseline(self) -> dict[str, Any]:
        return self.get_json("/api/security/baseline")

