# Initial state

The continuous simulator requires one finite initial_state value for every
VariableRole::State in the world and rejects both a missing state and an
identifier that is not a state. State identity is the canonical
lawsynth_core::Identifier, not display text.

The first trajectory row is the supplied state at config.start; it is not a
solver estimate. Initial values are never inferred from parameter defaults,
dataset rows, or zero. A caller that needs those policies MUST materialize the
complete state map before calling the simulator.
