# Interventions

The executable intervention types are Parameter(id) and Input(id), each with a
finite activation time and finite replacement value. They are converted to
scheduled parameter or input values in SimulationRequest. A parameter target
MUST exist in the world. An input target MUST exist and MUST NOT be a state
variable.

A change is active at its exact timestamp. When multiple accepted changes have
the same timestamp and identifier, deterministic sorting followed by assignment
makes the later item in the request sequence the effective value. Values persist
until superseded; there is no pulse duration type.

State shocks, law replacement, graph-edge interventions, and causal do-syntax
are outside the current executable contract.
