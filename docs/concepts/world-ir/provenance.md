# Provenance

`Dataset` requires a strictly increasing finite `TimeAxis` and aligned named numeric columns. It can batch and window those rows, expose a schema, and produce content fingerprints. Its Parquet reader decodes flat, required, uncompressed PLAIN numeric columns after validating the `PAR1` envelope and compact-Thrift metadata; compressed pages, dictionaries, nullable/repeated fields, and unsupported encodings fail explicitly.

Discovery results retain input profiling, preprocessing reports, candidate metrics, and optional bootstrap summaries. `DiscoveryCheckpoint` binds partial work to both a dataset fingerprint and a configuration fingerprint before it permits reuse.

The data layer exposes a content fingerprint based on the ordered time axis, column identifiers, values, and units. Preserve the source dataset, configuration, solver settings, and package version beside a selected world.

A `World` itself stores executable structure. It does not embed a complete experiment record, a signed evidence chain, or a source-data license.
