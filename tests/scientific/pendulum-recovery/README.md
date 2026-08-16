# Damped pendulum candidate-law case

This fixture uses `theta' = omega` and
`omega' = -damping * omega - (g / length) * sin(theta)`. The native CLI is run
with trigonometric features and its world is then executed. A generated
trajectory is evidence for software behavior only; parameter identifiability is
not asserted by this benchmark.
