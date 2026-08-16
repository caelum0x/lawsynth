# Resources

lawsynth-api-types validates identifiers for projects, datasets, worlds, runs,
and artifacts, and supplies descriptors/revisions rather than persistence.
SimulationRequest references a WorldRevision, validated TimeRange, seed, and a
nonempty unique list of output-variable names.

No endpoint paths, database schema, object store, upload protocol, or resource
ownership enforcement exists in this release. A service implementation MUST
define representation media types and map stored objects to these validated
types without weakening their constraints.
