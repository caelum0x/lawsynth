# World IR 0.1

World IR is the validated in-memory model consumed by the LawSynth simulator and bundle codec. Version 0.1 represents a deterministic scalar system as either a continuous `World` or a simultaneous-update `DiscreteWorld`. It is deliberately narrower than a general causal or hybrid model: a world contains variables, constant scalar parameters, and one scalar expression per state variable.

The normative implementation is `lawsynth-world`, with identifiers from `lawsynth-core`, expressions from `lawsynth-expr`, and dimensional checking from `lawsynth-units`. Public constructors validate the invariants in this directory; callers must not treat construction of a Rust struct by other means as a valid World IR artifact.

World IR has no stochastic laws, delays, algebraic constraints, learned weights, event actions, regime-selected laws, or serialization of interventions. Event, regime, and intervention types are useful runtime metadata, but are not part of the version-0.1 bundle payload.
