# Inputs and interventions

`SimulationRequest` supports finite initial states, parameter overrides, constant input values, and scheduled parameter/input values. Typed world interventions are translated into the appropriate scheduled change. At identical times, identifier order determines deterministic overwrite order.

Only parameter and non-state input targets are permitted. The simulation API does not intervene on equation structure, state resets, distributions, or hidden variables.
