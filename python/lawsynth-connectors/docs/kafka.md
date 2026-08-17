# Kafka connector

`KafkaConnector` (`lawsynth_connectors.kafka`) performs **finite** capture from a
Kafka topic with explicit bounds and offset provenance. It is not an unbounded
stream subscription: every read terminates on a message ceiling or an idle
timeout. The `confluent_kafka` driver is optional; without it the connector raises
`DependencyUnavailableError` for the `kafka` extra.

## Configuration

The read `resource` is the `bootstrap.servers` value. Required and optional inputs
come from the request/connector `options`:

- `group_id` (required) — consumer group id.
- `topic` (required) — topic to subscribe to.
- `max_messages` (default: `limit` or `config.max_rows`) — hard message ceiling,
  validated against `config.max_rows`; exceeding it raises `LimitExceededError`.
- `idle_timeout_seconds` (default: `config.timeout_seconds`) — capture stops after
  this long without a new message.
- `offset_reset` (default `earliest`), `security_protocol` (default `SASL_SSL`),
  `sasl_mechanism` (default `PLAIN`).

Auto-commit is disabled and partition EOF is enabled, so capture is deterministic
and does not silently advance a shared group's offsets.

## Credentials

SASL is enabled only when both `kafka_username` and `kafka_password` are present
in the credential chain; supplying just one raises `ConfigurationError`. The
secrets are `.reveal()`ed solely while building the consumer config and never
appear in logs or error details.

## Provenance

Captured records carry their Kafka offset metadata as snapshot provenance, so a
bounded capture can be described and, where the source allows, re-read against the
same offsets. Records flow through the standard bounded `DataBatch` pipeline with
fingerprints. Capabilities: `read`, `snapshots`, `streaming`.
