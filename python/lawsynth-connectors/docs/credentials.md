# Credentials

Connectors resolve secrets through a small, injectable provider chain so that no
secret is ever printed, logged, or stored in a global. The types live in
`lawsynth_connectors.credentials`.

## SecretValue

`SecretValue` wraps sensitive text and redacts itself everywhere except the one
narrow driver boundary that needs the raw value:

```python
from lawsynth_connectors.credentials import SecretValue

secret = SecretValue("pg-password")
str(secret)      # "[REDACTED]"
repr(secret)     # "SecretValue('[REDACTED]')"
secret.reveal()  # "pg-password"  -> only at the driver call site
```

An empty value raises `CredentialError`.

## Providers

All providers satisfy the `CredentialProvider` protocol (`get(name) -> SecretValue | None`):

- `StaticCredentialProvider` — explicit values for dependency injection and
  notebooks. `StaticCredentialProvider.from_strings({...})` wraps plain strings.
- `EnvironmentCredentialProvider` — reads a constrained namespace, default prefix
  `LAWSYNTH_CONNECTOR_`. A lookup for `kafka-password` reads
  `LAWSYNTH_CONNECTOR_KAFKA_PASSWORD`. Invalid names or prefixes raise
  `CredentialError`.
- `CredentialChain` — resolves providers in a deterministic, caller-controlled
  order and returns the first match.

## Usage in connectors

A connector receives its chain at construction and asks for named secrets when it
builds a driver config. For example the Kafka connector pairs
`kafka_username` / `kafka_password` and calls `.reveal()` only while assembling
the SASL client options. If a required secret is absent the connector raises a
`ConfigurationError` describing the missing name — never the value. Because
providers hold `SecretValue`, an accidental log of the chain or config still shows
`[REDACTED]`.
