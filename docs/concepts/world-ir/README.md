# World IR

LawSynth executes a `World` as named state variables, parameters, controls, and one law per state. A continuous world interprets each law as a derivative; a discrete world interprets it as the next state value. Both constructors reject duplicate identifiers, non-finite parameter values, unknown expression symbols, and a missing law for any state.

The Rust IR stores maps in identifier order. That gives dependency views and bundle output a stable order. Python's `build_world` validates the same public shape before it calls the native binding.

Use the World IR for autonomous scalar state updates. It does not model vector-valued states, spatial meshes, algebraic constraints, delayed history, or event-triggered resets as executable world behavior.
