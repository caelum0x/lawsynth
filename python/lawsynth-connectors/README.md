# LawSynth connectors

`lawsynth-connectors` provides bounded, reproducible ingestion adapters for the
LawSynth Python stack. Connectors normalize external sources into batches of
plain records while preserving source metadata, snapshot identity, and a stable
content fingerprint.

The package deliberately keeps third-party drivers optional. Importing the core
package never imports pandas, Arrow, database clients, cloud SDKs, or streaming
clients. A concrete adapter raises `DependencyUnavailableError` with the exact
extra to install when its driver is missing.

## Contract

Every connector:

- receives immutable `ConnectorConfig` limits;
- exposes explicit capabilities and health information;
- returns bounded `DataBatch` values rather than an unbounded iterator;
- redacts credentials and sensitive error details;
- records source/snapshot metadata and deterministic fingerprints;
- supports predictable cleanup through a context manager.

```python
from lawsynth_connectors import ConnectorConfig, ReadRequest, registry

connector = registry.create(
    ConnectorConfig(
        name="filesystem",
        batch_size=5_000,
        max_rows=100_000,
        options={"root": "./datasets"},
    )
)

with connector:
    batches = connector.read(ReadRequest(resource="observations.csv"))
    rows = [row for batch in batches for row in batch.records]
```

Adapters do not perform LawSynth domain inference. Their boundary ends after
safe acquisition, structural validation, and provenance capture; profiling and
world discovery remain responsibilities of the main SDK.
