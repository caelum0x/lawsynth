# Events and discontinuities

The executable simulation API supports scheduled input and parameter changes at specified times. It does not expose root-finding event callbacks, zero-crossing triggers, reset maps, hybrid automata, or user-provided event code.

Represent a known exogenous time change with an `--input-at` or `--parameter-at` assignment. For an event that depends on the simulated state, segment the analysis with an externally validated procedure and clearly label that procedure; do not claim it was handled by the engine.

The Python `Event` class is a validated marker for client-side metadata. Creating one does not install an event detector into native simulation. Keep detection logic and its tolerance in the calling application, then verify the resulting trajectories against the intended event semantics.
