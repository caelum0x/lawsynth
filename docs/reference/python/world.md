# Worlds

`build_world(states, parameters, equations, controls=())` validates the schema before calling native `World`. State variables must use role `state`, controls role `control`, identifiers cannot collide with one another or parameter names, and each state must have exactly one equation target.

The resulting object is a native continuous executable world. Equation syntax and semantic validation occur in the compiled layer. Python does not build discrete worlds, hybrid models, stochastic worlds, delay equations, or causal graphs.
