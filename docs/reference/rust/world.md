# `lawsynth-world`

`World` and `DiscreteWorld` combine named variables, finite parameters, and exactly one law per state. Variables have roles `State`, `Control`, `Exogenous`, `Observed`, `Latent`, or `Derived`; variables and parameters can carry parsed `Unit`s. Continuous laws describe derivatives, while discrete laws describe the next update.

Construction validates namespace uniqueness, targets, expression references, and time semantics before simulation or serialization. Worlds are mathematical executable IR, not arbitrary user programs; regimes, causal graphs, event logic, and uncertainty models are not part of this world contract.
