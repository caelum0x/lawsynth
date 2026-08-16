# Uncertainty lens boundary

The current CLI and Python simulation paths are deterministic. Bootstrap can be configured during discovery, but the interfaces do not provide a calibrated posterior, trajectory confidence band, stochastic simulator, or browser uncertainty service.

A host may display externally computed uncertainty artifacts only when their method, input models, random seeds, and coverage assumptions are preserved with the result. Label these as external analysis, not native LawSynth output, unless an implemented API actually returned them.

Avoid showing translucent bands by default: a visual encoding without an uncertainty model can mislead more than it informs. Make absence of calibrated uncertainty visible in the interface.
