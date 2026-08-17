"""S3 connector: URI parsing, decoding, and dependency degradation."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import DataValidationError, DependencyUnavailableError
from lawsynth_connectors.s3 import S3Connector, parse_s3_uri


def test_parse_s3_uri_valid() -> None:
    assert parse_s3_uri("s3://bucket/path/to/key.csv") == ("bucket", "path/to/key.csv")


@pytest.mark.parametrize(
    "uri",
    ["http://bucket/key", "s3://bucket", "s3:///key", "s3://bucket/"],
)
def test_parse_s3_uri_rejects_bad_forms(uri: str) -> None:
    with pytest.raises(DataValidationError):
        parse_s3_uri(uri)


def test_decode_csv_and_jsonl_and_json() -> None:
    csv_rows = S3Connector._decode(b"a,b\n1,2\n", "d.csv", "text/csv")
    assert csv_rows == [{"a": "1", "b": "2"}]
    jsonl_rows = S3Connector._decode(b'{"a": 1}\n{"a": 2}\n', "d.jsonl", None)
    assert jsonl_rows == [{"a": 1}, {"a": 2}]
    single = S3Connector._decode(b'{"a": 1}', "d.json", None)
    assert single == [{"a": 1}]


def test_decode_rejects_non_object_records() -> None:
    with pytest.raises(DataValidationError):
        S3Connector._decode(b"[1, 2, 3]", "d.json", None)
    with pytest.raises(DataValidationError):
        S3Connector._decode(b"\xff\xfe", "d.csv", "text/csv")


def test_capabilities() -> None:
    connector = registry.create(ConnectorConfig(name="s3"))
    caps = connector.capabilities
    assert caps.read and caps.snapshots and caps.projections


def test_missing_boto3_degrades_on_connect() -> None:
    pytest.importorskip
    connector = registry.create(ConnectorConfig(name="s3"))
    try:
        import boto3  # noqa: F401
    except ImportError:
        with pytest.raises(DependencyUnavailableError) as raised:
            connector.connect()
        assert raised.value.details["dependency"] == "boto3"
    else:  # pragma: no cover - exercised only when boto3 is installed
        connector.connect()
        connector.close()
