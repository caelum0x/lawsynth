# Equation explorer integration

Equation display should be derived from an inspected native bundle or discovery result, with state names, parameters, units, and source hash kept together. Show coefficients at a precision appropriate to validation evidence and preserve the machine-readable bundle as the authoritative artifact.

`chart-core` can render dependency-graph data models and trajectories, but it does not parse LawSynth bundles or typeset equations. A host application must implement a trusted bundle-to-view model conversion and test it against known bundles.

Do not permit an equation display field to become executable code. The current production interfaces do not accept arbitrary equations from a Studio client or load custom operators at runtime.
