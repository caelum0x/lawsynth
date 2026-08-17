# Errors

Every failure a connector raises is a `ConnectorError` or a subclass, defined in
`lawsynth_connectors.errors`. Errors carry machine-readable, redacted context so
they are safe to log and to return over an API.

## Shape

```python
try:
    connector.read(request)
except ConnectorError as exc:
    payload = exc.as_dict()
    # {"code": "...", "message": "...", "retryable": bool,
    #  "connector": "...", "details": {...}}
```

- `code` — a stable identifier (e.g. `connector_configuration`).
- `message` — a human-readable summary; never contains secret values.
- `retryable` — whether a caller may retry using the configured `RetryPolicy`.
- `details` — a mapping passed through `_safe_details`, which replaces any value
  whose key looks sensitive (`password`, `token`, `secret`, `api_key`,
  `access_key`, `credential`, `authorization`) with `[REDACTED]` and repr-encodes
  non-primitive values.

## Hierarchy

| Exception | Code | Raised when |
| --- | --- | --- |
| `ConnectorError` | `connector_error` | base class for all connector failures |
| `ConfigurationError` | `connector_configuration` | invalid config, request, or capability use |
| `CredentialError` | `connector_credentials` | missing or malformed secret |
| `DependencyUnavailableError` | `connector_dependency_unavailable` | optional driver not installed |
| `ConnectorConnectionError` | `connector_connection` | driver failed to connect |
| `QueryError` | `connector_query` | source rejected a query |
| `DataValidationError` | `connector_data_validation` | records failed structural validation |
| `ResourceNotFoundError` | `connector_resource_not_found` | resource or snapshot missing |
| `SnapshotNotFoundError` | `connector_snapshot_not_found` | specialization of not-found for snapshots |
| `LimitExceededError` | `connector_limit_exceeded` | read/write exceeded `max_rows` or `max_bytes` |

`DependencyUnavailableError` (aliased `UnsupportedCapabilityError`) always states
the exact extra to install, e.g. *"install lawsynth-connectors[arrow]"*. Driver
initialization failures inside `connect()` are wrapped as `ConnectorError` with
only the offending exception's type name in `details`, so the original message —
which may embed a DSN or token — never escapes.
