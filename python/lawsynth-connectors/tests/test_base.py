"""Lifecycle, bounded batching, validation, and config-bound behavior."""

from __future__ import annotations

from collections.abc import Iterable, Sequence

import pytest

from lawsynth_connectors import (
    BaseConnector,
    ConnectorCapabilities,
    ConnectorConfig,
    ConnectorState,
    DataBatch,
    ReadRequest,
    Record,
    RetryPolicy,
    WriteRequest,
)
from lawsynth_connectors.errors import (
    ConfigurationError,
    ConnectorError,
    LimitExceededError,
)


class _ListConnector(BaseConnector):
    """Minimal in-memory connector exercising the base machinery."""

    capabilities = ConnectorCapabilities(read=True, write=True)

    def __init__(self, config: ConnectorConfig, rows: Sequence[Record] = (), **kw: object) -> None:
        super().__init__(config, **kw)
        self._rows = list(rows)
        self.written: list[Record] = []
        self.write_batches = 0

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        return list(self._rows)

    def _write_records(self, request: WriteRequest, records, *, first_batch: bool) -> None:
        if first_batch:
            self.write_batches = 0
        self.written.extend(records)
        self.write_batches += 1


def _cfg(**kw: object) -> ConnectorConfig:
    return ConnectorConfig(name="filesystem", **kw)  # type: ignore[arg-type]


# --- ReadRequest validation -------------------------------------------------


def test_read_request_normalizes_and_freezes_fields() -> None:
    request = ReadRequest("data.csv", columns=["a", "b"], filters={"k": 1}, options={"x": 1})
    assert request.columns == ("a", "b")
    with pytest.raises(TypeError):
        request.filters["new"] = 2  # type: ignore[index]
    with pytest.raises(TypeError):
        request.options["new"] = 2  # type: ignore[index]


@pytest.mark.parametrize(
    "kwargs",
    [
        {"resource": "   "},
        {"resource": "x", "limit": 0},
        {"resource": "x", "offset": -1},
        {"resource": "x", "columns": ["a", "a"]},
        {"resource": "x", "time_column": "  "},
    ],
)
def test_read_request_rejects_invalid_arguments(kwargs: dict[str, object]) -> None:
    with pytest.raises(ConfigurationError):
        ReadRequest(**kwargs)  # type: ignore[arg-type]


def test_write_request_validation() -> None:
    request = WriteRequest("out.csv", mode="append", partition_by=["a"])
    assert request.partition_by == ("a",)
    with pytest.raises(ConfigurationError):
        WriteRequest("out.csv", mode="upsert")  # type: ignore[arg-type]
    with pytest.raises(ConfigurationError):
        WriteRequest("out.csv", partition_by=["a", "a"])
    with pytest.raises(ConfigurationError):
        WriteRequest("  ")


# --- DataBatch --------------------------------------------------------------


def test_data_batch_from_records_is_immutable_and_fingerprinted(simple_records) -> None:
    batch = DataBatch.from_records(simple_records, source="s")
    assert batch.row_count == 3
    assert batch.fingerprint.digest
    with pytest.raises(TypeError):
        batch.records[0]["a"] = 99  # type: ignore[index]


def test_data_batch_rejects_bad_index_and_source(simple_records) -> None:
    from lawsynth_connectors.fingerprints import fingerprint_records

    fp = fingerprint_records(simple_records)
    with pytest.raises(ValueError):
        DataBatch(records=simple_records, fingerprint=fp, source="s", index=-1)
    with pytest.raises(ValueError):
        DataBatch(records=simple_records, fingerprint=fp, source="")


# --- capabilities -----------------------------------------------------------


def test_capabilities_defaults_are_read_only() -> None:
    caps = ConnectorCapabilities()
    assert caps.read is True
    assert caps.write is False
    with pytest.raises(AttributeError):
        caps.read = False  # type: ignore[misc]


# --- lifecycle --------------------------------------------------------------


def test_lifecycle_transitions_and_context_manager() -> None:
    connector = _ListConnector(_cfg(), rows=[{"a": 1}])
    assert connector.state is ConnectorState.NEW
    with connector as opened:
        assert opened.state is ConnectorState.CONNECTED
    assert connector.state is ConnectorState.CLOSED


def test_read_before_connect_raises() -> None:
    connector = _ListConnector(_cfg(), rows=[{"a": 1}])
    with pytest.raises(ConnectorError):
        connector.read(ReadRequest("x"))


def test_closed_connector_cannot_reconnect() -> None:
    connector = _ListConnector(_cfg())
    connector.connect()
    connector.close()
    with pytest.raises(ConnectorError):
        connector.connect()


def test_unexpected_constructor_argument_rejected() -> None:
    with pytest.raises(TypeError):
        _ListConnector(_cfg(), rows=[], bogus=1)  # type: ignore[call-arg]


# --- bounded batching -------------------------------------------------------


def test_bounded_batches_respect_batch_size() -> None:
    rows = [{"a": i} for i in range(5)]
    connector = _ListConnector(_cfg(batch_size=2), rows=rows)
    with connector:
        batches = connector.read(ReadRequest("x"))
    assert [b.row_count for b in batches] == [2, 2, 1]
    assert [b.index for b in batches] == [0, 1, 2]


def test_max_rows_caps_total_output() -> None:
    rows = [{"a": i} for i in range(10)]
    connector = _ListConnector(_cfg(batch_size=3, max_rows=4), rows=rows)
    with connector:
        batches = connector.read(ReadRequest("x"))
    assert sum(b.row_count for b in batches) == 4


def test_request_limit_caps_output() -> None:
    rows = [{"a": i} for i in range(10)]
    connector = _ListConnector(_cfg(batch_size=3), rows=rows)
    with connector:
        assert len(connector.read_all(ReadRequest("x", limit=2))) == 2


def test_projection_selects_and_validates_columns() -> None:
    connector = _ListConnector(_cfg(), rows=[{"a": 1, "b": 2}])
    with connector:
        rows = connector.read_all(ReadRequest("x", columns=["a"]))
        assert rows == ({"a": 1},)
        with pytest.raises(ConfigurationError):
            connector.read(ReadRequest("x", columns=["missing"]))


def test_numeric_validation_runs_during_read(numeric_records) -> None:
    connector = _ListConnector(_cfg(), rows=numeric_records)
    with connector:
        rows = connector.read_all(ReadRequest("x", numeric=True, time_column="time"))
    assert len(rows) == 3


def test_max_bytes_limit_raises() -> None:
    rows = [{"a": "x" * 100} for _ in range(50)]
    connector = _ListConnector(_cfg(batch_size=1, max_bytes=10), rows=rows)
    with connector:
        with pytest.raises(LimitExceededError):
            connector.read(ReadRequest("x"))


# --- write ------------------------------------------------------------------


def test_write_returns_result_with_counts() -> None:
    connector = _ListConnector(_cfg(batch_size=2))
    rows = [{"a": i} for i in range(5)]
    with connector:
        result = connector.write(WriteRequest("out", mode="append"), rows)
    assert result.row_count == 5
    assert result.batch_count == 3
    assert result.fingerprint.digest


def test_write_rejected_when_capability_absent() -> None:
    class _ReadOnly(_ListConnector):
        capabilities = ConnectorCapabilities(read=True, write=False)

    connector = _ReadOnly(_cfg())
    with connector:
        with pytest.raises(ConfigurationError):
            connector.write(WriteRequest("out"), [{"a": 1}])


# --- health & inspect -------------------------------------------------------


def test_health_reports_connected_state() -> None:
    connector = _ListConnector(_cfg())
    assert connector.health().healthy is False
    with connector:
        status = connector.health()
    assert status.healthy is True
    assert status.connector == "filesystem"
    assert status.latency_seconds >= 0


def test_inspect_default_reports_existing_resource() -> None:
    connector = _ListConnector(_cfg())
    with connector:
        info = connector.inspect("thing")
    assert info.exists is True


# --- ConnectorConfig & RetryPolicy ------------------------------------------


def test_config_normalizes_name_and_freezes_options() -> None:
    config = ConnectorConfig(name="  FileSystem  ", options={"root": "."})
    assert config.name == "filesystem"
    with pytest.raises(TypeError):
        config.options["x"] = 1  # type: ignore[index]


@pytest.mark.parametrize(
    "kwargs",
    [
        {"name": ""},
        {"name": "ok", "batch_size": 0},
        {"name": "ok", "max_rows": 0},
        {"name": "ok", "max_bytes": 0},
        {"name": "ok", "timeout_seconds": 0},
    ],
)
def test_config_rejects_out_of_bound_values(kwargs: dict[str, object]) -> None:
    with pytest.raises(ConfigurationError):
        ConnectorConfig(**kwargs)  # type: ignore[arg-type]


def test_config_rejects_secret_option_keys() -> None:
    with pytest.raises(ConfigurationError):
        ConnectorConfig(name="s3", options={"secret_key": "abc"})


def test_config_option_typed_accessor() -> None:
    config = ConnectorConfig(name="http", options={"records_key": "data"})
    assert config.option("records_key", str) == "data"
    with pytest.raises(ConfigurationError):
        config.option("records_key", int)
    with pytest.raises(ConfigurationError):
        config.option("absent", str, required=True)


def test_config_from_mapping_rejects_unknown_fields() -> None:
    with pytest.raises(ConfigurationError):
        ConnectorConfig.from_mapping({"name": "http", "bogus": 1})
    config = ConnectorConfig.from_mapping({"name": "http", "batch_size": 7})
    assert config.batch_size == 7


def test_retry_policy_delay_growth_and_validation() -> None:
    policy = RetryPolicy(initial_delay_seconds=1.0, multiplier=2.0, maximum_delay_seconds=5.0)
    assert policy.delay_for(0) == 1.0
    assert policy.delay_for(1) == 2.0
    assert policy.delay_for(10) == 5.0
    with pytest.raises(ConfigurationError):
        RetryPolicy(attempts=0)
    with pytest.raises(ValueError):
        policy.delay_for(-1)
