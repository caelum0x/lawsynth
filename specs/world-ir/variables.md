# Variables

Variables declare the symbols a world may expose. `State` variables are dynamic and must have one law. `Control`, `Exogenous`, `Observed`, `Latent`, and `Derived` are declarative roles in 0.1: they can be read by expressions but must not be law targets. The core validator does not infer roles or add equations for them.

An optional `Unit` attaches a scale and a seven-base SI dimension. Unit absence is meaningful: it is not dimensionless. When checking a law for a unit-bearing state, every referenced symbol needs a known unit. Use the literal dimensionless unit `1` when that is the intended quantity.
