# Determinism

For identical dataset bits and identical configuration values, the core
pipeline uses canonical maps, fixed feature enumeration, `total_cmp` where
ordering floating scores, deterministic sparse solvers, and stable Pareto
iteration. Dataset and configuration fingerprints bind a checkpoint to those
inputs.

Bootstrap resampling is deterministic moving-block sampling driven by the
configured `u64` seed and LawSynth's local SplitMix64 generator. Candidate
ranking breaks exact metric ties by original index. Symbolic selection breaks
equal calibrated errors by canonical expression text.

This is an in-process reproducibility contract. It does not claim bitwise
identity across all CPU architectures, compiler/libm versions, or changed
dependency versions; floating-point fitting remains sensitive to those
execution environments.
