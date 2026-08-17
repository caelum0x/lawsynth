"""Sandboxed local filesystem connector: formats, confinement, and writes."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, WriteRequest, registry
from lawsynth_connectors.errors import (
    ConfigurationError,
    DataValidationError,
    ResourceNotFoundError,
)

from .conftest import records_of


def _connector(root: Path, **cfg: object):
    return registry.create(ConnectorConfig(name="filesystem", options={"root": str(root)}, **cfg))


def test_missing_root_raises_on_connect(tmp_path: Path) -> None:
    connector = registry.create(
        ConnectorConfig(name="filesystem", options={"root": str(tmp_path / "nope")})
    )
    with pytest.raises(ResourceNotFoundError):
        connector.connect()


def test_read_csv_with_projection(tmp_path: Path) -> None:
    (tmp_path / "d.csv").write_text("a,b,c\n1,2,3\n4,5,6\n", encoding="utf-8")
    with _connector(tmp_path, batch_size=1) as connector:
        rows = records_of(connector.read(ReadRequest("d.csv", columns=["a", "c"])))
    assert rows == [{"a": "1", "c": "3"}, {"a": "4", "c": "6"}]


def test_read_tsv(tmp_path: Path) -> None:
    (tmp_path / "d.tsv").write_text("a\tb\n1\t2\n", encoding="utf-8")
    with _connector(tmp_path) as connector:
        rows = records_of(connector.read(ReadRequest("d.tsv")))
    assert rows == [{"a": "1", "b": "2"}]


def test_read_jsonl_with_offset_and_limit(tmp_path: Path) -> None:
    lines = [json.dumps({"i": i}) for i in range(5)]
    (tmp_path / "d.jsonl").write_text("\n".join(lines) + "\n", encoding="utf-8")
    with _connector(tmp_path) as connector:
        rows = records_of(connector.read(ReadRequest("d.jsonl", offset=1, limit=2)))
    assert rows == [{"i": 1}, {"i": 2}]


def test_read_json_records_envelope(tmp_path: Path) -> None:
    (tmp_path / "d.json").write_text(
        json.dumps({"records": [{"a": 1}, {"a": 2}]}), encoding="utf-8"
    )
    with _connector(tmp_path) as connector:
        rows = records_of(connector.read(ReadRequest("d.json")))
    assert rows == [{"a": 1}, {"a": 2}]


def test_unsupported_format_rejected(tmp_path: Path) -> None:
    (tmp_path / "d.xyz").write_text("nope", encoding="utf-8")
    with _connector(tmp_path) as connector:
        with pytest.raises(ConfigurationError):
            connector.read(ReadRequest("d.xyz"))


def test_absolute_and_escaping_paths_rejected(tmp_path: Path) -> None:
    (tmp_path / "d.jsonl").write_text('{"a": 1}\n', encoding="utf-8")
    with _connector(tmp_path) as connector:
        with pytest.raises(ConfigurationError):
            connector.read(ReadRequest("/etc/passwd"))
        with pytest.raises(ConfigurationError):
            connector.read(ReadRequest("../outside.jsonl"))


def test_missing_resource_raises(tmp_path: Path) -> None:
    with _connector(tmp_path) as connector:
        with pytest.raises(ResourceNotFoundError):
            connector.read(ReadRequest("absent.jsonl"))


def test_invalid_jsonl_line_raises(tmp_path: Path) -> None:
    (tmp_path / "d.jsonl").write_text("{not json}\n", encoding="utf-8")
    with _connector(tmp_path) as connector:
        with pytest.raises(DataValidationError):
            connector.read(ReadRequest("d.jsonl"))


def test_write_then_read_roundtrip_jsonl(tmp_path: Path) -> None:
    rows = [{"a": 1, "b": 2}, {"a": 3, "b": 4}]
    with _connector(tmp_path) as connector:
        result = connector.write(WriteRequest("out.jsonl", mode="replace"), rows)
        assert result.row_count == 2
        read_back = records_of(connector.read(ReadRequest("out.jsonl")))
    assert read_back == rows


def test_write_error_mode_refuses_existing(tmp_path: Path) -> None:
    (tmp_path / "out.csv").write_text("a\n1\n", encoding="utf-8")
    with _connector(tmp_path) as connector:
        with pytest.raises(ConfigurationError):
            connector.write(WriteRequest("out.csv", mode="error"), [{"a": 9}])


def test_inspect_reports_fingerprint_and_absence(tmp_path: Path) -> None:
    (tmp_path / "d.csv").write_text("a\n1\n", encoding="utf-8")
    with _connector(tmp_path) as connector:
        present = connector.inspect("d.csv")
        assert present.exists and present.snapshot
        assert connector.inspect("absent.csv").exists is False


def test_capabilities_declared() -> None:
    connector = registry.create(ConnectorConfig(name="filesystem", options={"root": "."}))
    caps = connector.capabilities
    assert caps.read and caps.write and caps.snapshots and caps.projections
