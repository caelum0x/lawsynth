"""HTTP connector URL validation, SSRF guard, and header safety.

The SSRF guard is the load-bearing security control here.  These tests assert
its *documented* behavior rather than fighting it:

* By default the guard rejects any URL that resolves to a non-global address
  (loopback ``127.0.0.1`` and other private ranges).
* The guard is only bypassed by the explicit ``allow_private_network`` opt-in,
  and only then does an end-to-end read of a loopback source succeed.
* An ``allowed_hosts`` allowlist is enforced before any DNS resolution.
"""

from __future__ import annotations

from urllib.parse import urlparse

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import ConfigurationError, DataValidationError
from lawsynth_connectors.http import HttpConnector

from .conftest import records_of


def _http(**options: object) -> HttpConnector:
    connector = registry.create(ConnectorConfig(name="http", options=options))
    assert isinstance(connector, HttpConnector)
    return connector


# --- SSRF guard: the documented default is to REJECT loopback/private ------


def test_ssrf_guard_rejects_loopback_by_default() -> None:
    connector = _http()
    with pytest.raises(DataValidationError, match="non-public address"):
        connector._validate_url("http://127.0.0.1:8000/data.csv")


def test_ssrf_guard_rejects_private_range_by_default() -> None:
    connector = _http()
    # 10.0.0.0/8 is private; guard resolves it to a non-global address.
    with pytest.raises(DataValidationError):
        connector._validate_url("http://10.0.0.1/data.csv")


def test_allow_private_network_opt_in_permits_loopback_validation() -> None:
    connector = _http(allow_private_network=True)
    url = "http://127.0.0.1:8000/data.csv"
    assert connector._validate_url(url) == url


def test_allow_private_network_must_be_boolean() -> None:
    connector = _http(allow_private_network="yes")
    with pytest.raises(ConfigurationError):
        connector._validate_url("http://127.0.0.1/data.csv")


def test_allowed_hosts_enforced_before_dns() -> None:
    connector = _http(allowed_hosts=["example.com"])
    with pytest.raises(DataValidationError, match="allowlisted"):
        connector._validate_url("http://127.0.0.1/data.csv")


# --- URL shape validation ---------------------------------------------------


def test_non_http_scheme_rejected() -> None:
    connector = _http(allow_private_network=True)
    with pytest.raises(DataValidationError):
        connector._validate_url("ftp://127.0.0.1/data")


def test_url_with_embedded_credentials_rejected() -> None:
    connector = _http(allow_private_network=True)
    with pytest.raises(DataValidationError):
        connector._validate_url("http://user:pass@127.0.0.1/data")


def test_relative_url_rejected() -> None:
    connector = _http(allow_private_network=True)
    with pytest.raises(DataValidationError):
        connector._validate_url("/just/a/path")


# --- header safety ----------------------------------------------------------


def test_sensitive_headers_from_options_are_rejected() -> None:
    connector = _http(headers={"Authorization": "Bearer x"})
    with pytest.raises(ConfigurationError):
        connector._headers()


def test_bearer_token_sourced_from_credentials() -> None:
    from lawsynth_connectors.credentials import CredentialChain, StaticCredentialProvider

    creds = CredentialChain((StaticCredentialProvider.from_strings({"http_bearer_token": "sekret"}),))
    connector = registry.create(ConnectorConfig(name="http"), credentials=creds)
    headers = connector._headers()  # type: ignore[attr-defined]
    assert headers["Authorization"] == "Bearer sekret"


def test_custom_headers_forwarded() -> None:
    connector = _http(headers={"X-Trace": "abc"})
    assert connector._headers()["X-Trace"] == "abc"


def test_http_capabilities() -> None:
    connector = _http()
    caps = connector.capabilities
    assert caps.read and caps.snapshots and caps.projections
    assert caps.write is False


# --- end-to-end: the opt-in actually permits a loopback read ----------------


def test_loopback_read_succeeds_only_with_opt_in(local_csv_server: str) -> None:
    host = urlparse(local_csv_server).hostname
    connector = registry.create(
        ConnectorConfig(
            name="http",
            batch_size=1,
            options={"allow_private_network": True, "allowed_hosts": [host]},
        )
    )
    with connector:
        rows = records_of(connector.read(ReadRequest(local_csv_server)))
    assert rows == [{"time": "0", "x": "1.25", "y": "2"}, {"time": "1", "x": "3.5", "y": "4"}]


def test_loopback_read_blocked_without_opt_in(local_csv_server: str) -> None:
    connector = registry.create(ConnectorConfig(name="http"))
    with connector:
        with pytest.raises(DataValidationError):
            connector.read(ReadRequest(local_csv_server))
