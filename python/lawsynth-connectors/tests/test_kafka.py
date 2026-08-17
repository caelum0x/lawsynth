"""Kafka connector: consumer config, capabilities, dependency degradation."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.credentials import CredentialChain, StaticCredentialProvider
from lawsynth_connectors.errors import ConfigurationError, DependencyUnavailableError
from lawsynth_connectors.kafka import KafkaConnector


def _kafka(credentials: CredentialChain | None = None, **options: object) -> KafkaConnector:
    connector = registry.create(
        ConnectorConfig(name="kafka", options=options),
        credentials=credentials or CredentialChain(()),
    )
    assert isinstance(connector, KafkaConnector)
    return connector


def test_consumer_config_requires_group_id() -> None:
    connector = _kafka()
    with pytest.raises(ConfigurationError):
        connector._consumer_config(ReadRequest("localhost:9092"))


def test_consumer_config_builds_bounded_defaults() -> None:
    connector = _kafka(group_id="g1")
    config = connector._consumer_config(ReadRequest("localhost:9092"))
    assert config["bootstrap.servers"] == "localhost:9092"
    assert config["group.id"] == "g1"
    assert config["enable.auto.commit"] is False
    assert config["enable.partition.eof"] is True
    assert config["auto.offset.reset"] == "earliest"


def test_consumer_config_requires_paired_credentials() -> None:
    creds = CredentialChain((StaticCredentialProvider.from_strings({"kafka_username": "u"}),))
    connector = _kafka(credentials=creds, group_id="g1")
    with pytest.raises(ConfigurationError):
        connector._consumer_config(ReadRequest("localhost:9092"))


def test_consumer_config_injects_sasl_when_paired() -> None:
    creds = CredentialChain(
        (StaticCredentialProvider.from_strings({"kafka_username": "u", "kafka_password": "p"}),)
    )
    connector = _kafka(credentials=creds, group_id="g1")
    config = connector._consumer_config(ReadRequest("localhost:9092"))
    assert config["security.protocol"] == "SASL_SSL"
    assert config["sasl.username"] == "u"
    assert config["sasl.password"] == "p"


def test_inspect_marks_bounded_capture() -> None:
    connector = _kafka(group_id="g1")
    with connector:
        info = connector.inspect("localhost:9092")
    assert info.kind == "stream"
    assert info.metadata["bounded_capture_required"] is True


def test_capabilities() -> None:
    connector = _kafka(group_id="g1")
    caps = connector.capabilities
    assert caps.read and caps.snapshots and caps.streaming


def test_missing_confluent_kafka_degrades_on_read() -> None:
    connector = _kafka(group_id="g1", topic="events")
    try:
        import confluent_kafka  # noqa: F401
    except ImportError:
        with connector:
            with pytest.raises(DependencyUnavailableError) as raised:
                connector.read(ReadRequest("localhost:9092", options={"topic": "events"}))
        assert raised.value.details["dependency"] == "confluent_kafka"
    else:  # pragma: no cover - only when the driver is installed
        pytest.skip("confluent_kafka installed; live broker not available in tests")
