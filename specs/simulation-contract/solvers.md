# Solvers

Continuous execution uses fixed-step classical fourth-order Runge-Kutta through
lawsynth_sim::simulate. It evaluates the World IR's continuous laws over a
deterministic BTreeMap state ordering. The engine stops with SimulationError if
an expression fails or produces a non-finite state.

Discrete execution uses simulate_discrete and evaluates all next-state laws
against the same prior state, then commits the transition simultaneously.

No adaptive step control, stiffness detection, error tolerance, implicit ODE
solver, delay solver, or solver-selection parameter exists in this release.
Callers MUST NOT interpret step as an accuracy guarantee.
