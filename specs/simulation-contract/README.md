# Simulation contract

This contract describes the executable, local simulation boundary implemented by
lawsynth-sim and the World IR in lawsynth-world. A continuous World is
integrated with classical RK4; a DiscreteWorld is evaluated as simultaneous
recurrences. This is an in-process Rust contract, not a network API.

Every accepted request has a finite, strictly increasing time domain and an
explicit value for every state variable. The engine rejects unknown identifiers,
non-finite values, invalid intervention times, and non-finite results.

Implemented surface: continuous RK4, discrete recurrence execution, constant and
scheduled parameter/input changes, typed Intervention, and dense trajectories.
Event-root solving, adaptive solvers, equation/edge interventions, delay solving,
and remote execution are not exposed by this contract.
