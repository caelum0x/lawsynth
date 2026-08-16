# Shared structure boundary

`RegimeLawBook` maps each integer regime to one affine law `intercept + slope × x`. It validates finite coefficients and evaluates only the selected law.

There is no shared-support sparse solver, parameter tying, or joint symbolic structure learner across segments. Any claim of shared discovered structure must come from an external fitting procedure.
