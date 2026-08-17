"""Credential providers keep secrets out of logs and repr output."""

from __future__ import annotations

import pytest

from lawsynth_connectors.credentials import (
    EMPTY_CREDENTIALS,
    CredentialChain,
    EnvironmentCredentialProvider,
    SecretValue,
    StaticCredentialProvider,
)
from lawsynth_connectors.errors import CredentialError


def test_secret_value_redacts_repr_and_str_but_reveals_at_boundary() -> None:
    secret = SecretValue("super-secret-token")
    assert repr(secret) == "SecretValue('[REDACTED]')"
    assert str(secret) == "[REDACTED]"
    assert "super-secret-token" not in repr(secret)
    assert "super-secret-token" not in f"{secret}"
    assert secret.reveal() == "super-secret-token"


def test_secret_value_rejects_empty() -> None:
    with pytest.raises(CredentialError):
        SecretValue("")


def test_static_provider_from_strings_and_lookup() -> None:
    provider = StaticCredentialProvider.from_strings({"token": "abc"})
    assert provider.get("token").reveal() == "abc"  # type: ignore[union-attr]
    assert provider.get("missing") is None


def test_environment_provider_reads_prefixed_namespace(monkeypatch) -> None:
    provider = EnvironmentCredentialProvider()
    monkeypatch.setenv("LAWSYNTH_CONNECTOR_APIKEY", "value")
    assert provider.get("apikey").reveal() == "value"  # type: ignore[union-attr]
    assert provider.get("unset") is None


def test_environment_provider_rejects_bad_names_and_prefix(monkeypatch) -> None:
    provider = EnvironmentCredentialProvider()
    with pytest.raises(CredentialError):
        provider.get("bad name!")
    # The guard requires strictly alphanumeric names (underscores are rejected).
    with pytest.raises(CredentialError):
        provider.get("api_key")
    with pytest.raises(CredentialError):
        EnvironmentCredentialProvider(prefix="")


def test_credential_chain_resolves_in_order() -> None:
    first = StaticCredentialProvider.from_strings({"token": "one"})
    second = StaticCredentialProvider.from_strings({"token": "two", "other": "x"})
    chain = CredentialChain((first, second))
    assert chain.get("token").reveal() == "one"  # type: ignore[union-attr]
    assert chain.get("other").reveal() == "x"  # type: ignore[union-attr]
    assert chain.get("none") is None


def test_credential_chain_require_raises_when_absent() -> None:
    with pytest.raises(CredentialError) as raised:
        EMPTY_CREDENTIALS.require("token", connector="http")
    # The credential name is redacted in error details by the error taxonomy.
    assert raised.value.details["credential"] == "[REDACTED]"
    assert raised.value.connector == "http"


def test_provider_protocol_runtime_checkable() -> None:
    from lawsynth_connectors.credentials import CredentialProvider

    assert isinstance(StaticCredentialProvider({}), CredentialProvider)
