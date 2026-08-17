"""Postgres connector: connection arguments, capabilities, dependency guard."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.credentials import CredentialChain, StaticCredentialProvider
from lawsynth_connectors.errors import DependencyUnavailableError
from lawsynth_connectors.postgres import PostgresConnector


def _postgres(credentials: CredentialChain | None = None, **options: object) -> PostgresConnector:
    connector = registry.create(
        ConnectorConfig(name="postgres", options=options),
        credentials=credentials or CredentialChain(()),
    )
    assert isinstance(connector, PostgresConnector)
    return connector


def test_connection_arguments_defaults() -> None:
    connector = _postgres()
    args = connector._connection_arguments()
    assert args["application_name"] == "lawsynth-connectors"
    assert args["connect_timeout"] >= 1
    assert "password" not in args


def test_connection_arguments_reveal_password_at_boundary() -> None:
    creds = CredentialChain(
        (StaticCredentialProvider.from_strings({"postgres_password": "s3cr3t"}),)
    )
    connector = _postgres(credentials=creds, application_name="tests")
    args = connector._connection_arguments()
    assert args["password"] == "s3cr3t"
    assert args["application_name"] == "tests"


def test_capabilities() -> None:
    connector = _postgres()
    caps = connector.capabilities
    assert caps.read and caps.predicates and caps.projections and caps.transactions


def test_missing_psycopg_degrades_on_read() -> None:
    connector = _postgres(dsn="postgresql://localhost/db")
    try:
        import psycopg  # noqa: F401
    except ImportError:
        with connector:
            with pytest.raises(DependencyUnavailableError) as raised:
                connector.read(
                    ReadRequest("db", options={"query": "SELECT 1"})
                )
        assert raised.value.details["dependency"] == "psycopg"
    else:  # pragma: no cover - only when the driver is installed
        pytest.skip("psycopg installed; live server not available in tests")
