# `lawsynth-sim`

`simulate(world, SimulationConfig, SimulationRequest)` executes a continuous executable `World`; `simulate_discrete` executes a `DiscreteWorld` with `DiscreteSimulationConfig`. Requests provide initial values, parameter/input overrides, and time-stamped parameter/input schedules. Results contain time and state trajectories after solver and finite-value validation.

The crate contains explicit module boundaries for discrete, hybrid, and SDE semantics, but only the compiled, validated deterministic paths are a supported production interface. Do not claim event localization, hybrid transitions, delay equations, stochastic sampling, or arbitrary solver/plugin selection unless the concrete API accepts it.
