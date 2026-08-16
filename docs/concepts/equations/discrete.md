# Discrete laws

`DiscreteLaw` maps the current context to a next state value. `simulate_discrete` evaluates the compiled world and applies state updates simultaneously, preventing a map’s result from depending on hash-map iteration order.

Use a discrete world when the law itself defines the update interval. Preserve the sampling interpretation with the model because the IR does not attach a timestep unit to a discrete law.

Discrete simulation does not infer a continuous counterpart or interpolate between updates.
