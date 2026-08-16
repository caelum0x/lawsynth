# Lotka–Volterra candidate-law case

This case generates prey/predator observations from the classical two-state
Lotka–Volterra ODE, uses the public `lawsynth discover` CLI with a quadratic
feature library, and simulates the resulting world through `lawsynth simulate`.
It verifies that the native pipeline produces an executable candidate. It does
not call a single trajectory proof of biological mechanism or exact coefficient
recovery.
