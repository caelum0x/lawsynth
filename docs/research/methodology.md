# Methodology

LawSynth discovery is a constrained inference workflow, not an unconstrained explanation engine. The continuous path validates an increasing finite time axis and aligned finite numeric columns; estimates derivatives; evaluates a finite candidate feature library; fits coefficients; and ranks the resulting laws by fit and complexity. The selected laws are assembled into a typed World IR and can be simulated from declared initial conditions.

Choose state columns, allowed terms, differentiation method, sparse solver settings, split strategy, and acceptance metric before inspecting candidates. A defensible evaluation holds out whole intervals or experiments, simulates from observed initial state, and compares the rollout with the withheld observations. Do not use training residuals as validation evidence.

The current default solver path is deterministic. Seeded bootstrap and stochastic simulation paths require recording their seed. Neither a small residual nor a sparse expression establishes causality, identifiability, unit consistency, or validity outside the measured regime.
