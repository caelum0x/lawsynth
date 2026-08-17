"""Shared fixtures and import-path wiring for the connector test suite."""

from __future__ import annotations

import sys
import threading
from collections.abc import Iterator
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest

# Make ``src/lawsynth_connectors`` importable without an editable install so the
# suite runs both under ``PYTHONPATH=src`` and from a bare checkout.
_SRC = Path(__file__).resolve().parents[1] / "src"
if _SRC.is_dir() and str(_SRC) not in sys.path:
    sys.path.insert(0, str(_SRC))

from lawsynth_connectors import ConnectorConfig, DataBatch  # noqa: E402


@pytest.fixture
def numeric_records() -> list[dict[str, float | int]]:
    """A rectangular time-series dataset accepted by numeric validation."""
    return [
        {"time": 0, "x": 1.0, "y": 2.0},
        {"time": 1, "x": 3.0, "y": 4.0},
        {"time": 2, "x": 5.0, "y": 6.0},
    ]


@pytest.fixture
def simple_records() -> list[dict[str, object]]:
    return [{"a": 1, "b": "one"}, {"a": 2, "b": "two"}, {"a": 3, "b": "three"}]


@pytest.fixture
def make_config():
    """Factory building a validated :class:`ConnectorConfig`."""

    def _factory(name: str = "filesystem", **overrides: object) -> ConnectorConfig:
        return ConnectorConfig(name=name, **overrides)  # type: ignore[arg-type]

    return _factory


def records_of(batches: tuple[DataBatch, ...]) -> list[dict[str, object]]:
    """Flatten batches into plain dicts for equality assertions."""
    return [dict(row) for batch in batches for row in batch.records]


class _CsvHandler(BaseHTTPRequestHandler):
    body = b"time,x,y\n0,1.25,2\n1,3.5,4\n"

    def do_GET(self) -> None:  # noqa: N802 - stdlib framework spelling
        self.send_response(200)
        self.send_header("Content-Type", "text/csv")
        self.send_header("Content-Length", str(len(self.body)))
        self.send_header("ETag", "local-fixture")
        self.end_headers()
        self.wfile.write(self.body)

    def log_message(self, *_: object) -> None:
        return


@pytest.fixture
def local_csv_server() -> Iterator[str]:
    """Serve a small CSV from loopback; skip if the sandbox forbids binding."""
    try:
        server = ThreadingHTTPServer(("127.0.0.1", 0), _CsvHandler)
    except (PermissionError, OSError):
        pytest.skip("sandbox does not permit binding a local HTTP port")
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}/observations.csv"
    finally:
        server.shutdown()
        thread.join()
        server.server_close()
