# Algorithm version

`EngineConfig::default()` stores `CURRENT_ENGINE_VERSION`, currently the
semantic engine version defined by `lawsynth-core`. It is useful provenance but
does not identify a source revision, algorithm configuration, model grammar,
or dependency implementation.

Record the engine version with the repository commit and all relevant crate or
package versions. Results may change across versions even if the public API is
compatible; a semantic version is not a numerical-equivalence promise.
