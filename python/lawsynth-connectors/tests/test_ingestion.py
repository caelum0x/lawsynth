"""End-to-end connector checks against real local transports and SQLite."""

from __future__ import annotations

import json
import sqlite3
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import (
    ConfigurationError,
    DataValidationError,
    DependencyUnavailableError,
)


class _RecordsHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:  # noqa: N802 - HTTP framework spelling
        body = b"time,x,y\n0,1.25,2\n1,3.5,4\n"
        self.send_response(200)
        self.send_header("Content-Type", "text/csv")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("ETag", "local-fixture")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_: object) -> None:
        return


@pytest.fixture
def http_url() -> str:
    try:
        server = ThreadingHTTPServer(("127.0.0.1", 0), _RecordsHandler)
    except PermissionError:
        pytest.skip("sandbox does not permit binding a local HTTP port")
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}/observations.csv"
    finally:
        server.shutdown()
        thread.join()
        server.server_close()


def _records(batches: object) -> list[dict[str, object]]:
    return [dict(row) for batch in batches for row in batch.records]  # type: ignore[union-attr]


def test_filesystem_jsonl_numeric_batches_and_root_confinement(tmp_path: Path) -> None:
    (tmp_path / "observations.jsonl").write_text(
        "\n".join(json.dumps(row) for row in ({"time": 0, "x": 1.0}, {"time": 1, "x": 2.0}, {"time": 2, "x": 3.0})) + "\n",
        encoding="utf-8",
    )
    connector = registry.create(ConnectorConfig(name="filesystem", batch_size=2, options={"root": str(tmp_path)}))
    with connector:
        batches = connector.read(ReadRequest("observations.jsonl", numeric=True, time_column="time"))
        assert [batch.row_count for batch in batches] == [2, 1]
        assert _records(batches)[1]["x"] == 2.0
        assert batches[0].fingerprint.digest
        assert connector.inspect("observations.jsonl").snapshot
        with pytest.raises(ConfigurationError):
            connector.read(ReadRequest("../outside.jsonl"))
    assert connector.state.value == "closed"


def test_http_csv_is_streamed_coerced_and_numeric_validated(http_url: str) -> None:
    connector = registry.create(ConnectorConfig(name="http", batch_size=1, max_bytes=1024))
    with connector:
        batches = connector.read(ReadRequest(http_url, numeric=True, time_column="time"))
    assert _records(batches) == [{"time": 0, "x": 1.25, "y": 2}, {"time": 1, "x": 3.5, "y": 4}]
    assert batches[0].snapshot["etag"] == "local-fixture"


def test_sqlite_read_only_query_is_batched_and_write_sql_is_rejected(tmp_path: Path) -> None:
    database = tmp_path / "observations.sqlite"
    with sqlite3.connect(database) as connection:
        connection.execute("create table observations (time integer, x real)")
        connection.executemany("insert into observations values (?, ?)", [(0, 1.0), (1, 2.5), (2, 4.0)])
    connector = registry.create(ConnectorConfig(name="sqlite", batch_size=2))
    with connector:
        batches = connector.read(ReadRequest(str(database), numeric=True, time_column="time", options={"query": "select time, x from observations order by time"}))
        assert _records(batches)[-1] == {"time": 2, "x": 4.0}
        with pytest.raises(DataValidationError):
            connector.read(ReadRequest(str(database), options={"query": "delete from observations"}))


def test_missing_optional_backend_is_explicit_not_a_fallback() -> None:
    connector = registry.create(ConnectorConfig(name="duckdb"))
    with connector:
        with pytest.raises(DependencyUnavailableError) as raised:
            connector.read(ReadRequest(":memory:", options={"query": "select 1"}))
    assert raised.value.details["dependency"] == "duckdb"
