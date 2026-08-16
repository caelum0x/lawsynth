# Plan identity

There is no implemented `DiscoveryPlan` serialization or plan-hash API in the
current engine. `EngineConfig` carries a seed, version, and four resource
limits, but not a canonical description of preprocessing, feature generation,
scoring, or optimizer choices.

An integrator that needs a plan digest must define a canonical, versioned
encoding outside this API, include every behavior-affecting option, and hash
its exact bytes with a cryptographic function. Do not hash display text or
Rust debug output: neither is a stability contract.
