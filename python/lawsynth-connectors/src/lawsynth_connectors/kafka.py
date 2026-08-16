"""Finite Kafka capture with explicit bounds and offset provenance."""

from __future__ import annotations

import json
import time
from collections.abc import Iterable

from ._optional import dependency
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import ConfigurationError, DataValidationError, LimitExceededError


class KafkaConnector(BaseConnector):
    capabilities = ConnectorCapabilities(
        read=True,
        snapshots=True,
        streaming=True,
    )

    def _consumer_config(self, request: ReadRequest) -> dict[str, object]:
        group_id = request.options.get("group_id", self.config.options.get("group_id"))
        if not group_id:
            raise ConfigurationError("Kafka group_id is required")
        config: dict[str, object] = {
            "bootstrap.servers": request.resource,
            "group.id": str(group_id),
            "auto.offset.reset": str(request.options.get("offset_reset", "earliest")),
            "enable.auto.commit": False,
            "enable.partition.eof": True,
        }
        username = self.credentials.get("kafka_username")
        password = self.credentials.get("kafka_password")
        if username or password:
            if not (username and password):
                raise ConfigurationError("Kafka username and password must be paired")
            config.update(
                {
                    "security.protocol": str(
                        self.config.options.get("security_protocol", "SASL_SSL")
                    ),
                    "sasl.mechanism": str(
                        self.config.options.get("sasl_mechanism", "PLAIN")
                    ),
                    "sasl.username": username.reveal(),
                    "sasl.password": password.reveal(),
                }
            )
        return config

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        confluent = dependency("confluent_kafka", extra="kafka", connector="kafka")
        topic = request.options.get("topic")
        if not isinstance(topic, str) or not topic.strip():
            raise ConfigurationError("Kafka topic is required")
        maximum = int(request.options.get("max_messages", request.limit or self.config.max_rows))
        if not 1 <= maximum <= self.config.max_rows:
            raise LimitExceededError("Kafka max_messages exceeds connector limit")
        idle_timeout = float(
            request.options.get("idle_timeout_seconds", self.config.timeout_seconds)
        )
        if idle_timeout <= 0:
            raise ConfigurationError("Kafka idle timeout must be positive")

        consumer = confluent.Consumer(self._consumer_config(request))
        consumer.subscribe([topic])
        received = 0
        idle_since = time.monotonic()
        try:
            while received < maximum and time.monotonic() - idle_since < idle_timeout:
                message = consumer.poll(min(1.0, idle_timeout))
                if message is None:
                    continue
                if message.error():
                    if message.error().code() == confluent.KafkaError._PARTITION_EOF:
                        continue
                    raise DataValidationError(f"Kafka consumer error: {message.error()}")
                idle_since = time.monotonic()
                try:
                    payload = json.loads(message.value().decode("utf-8"))
                except (AttributeError, UnicodeDecodeError, json.JSONDecodeError) as exc:
                    raise DataValidationError("Kafka value must be a JSON object") from exc
                if not isinstance(payload, dict):
                    raise DataValidationError("Kafka value must be a JSON object")
                payload = dict(payload)
                if request.options.get("include_metadata", False):
                    payload["_kafka"] = {
                        "topic": message.topic(),
                        "partition": message.partition(),
                        "offset": message.offset(),
                        "timestamp": message.timestamp()[1],
                    }
                received += 1
                yield payload
        finally:
            consumer.close()

    def _inspect(self, resource: str) -> ResourceInfo:
        return ResourceInfo(
            resource,
            True,
            kind="stream",
            metadata={"bounded_capture_required": True},
        )
