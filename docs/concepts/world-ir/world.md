# World construction

Create a `World` from `Variable`, `Parameter`, and `ContinuousLaw` values. State variables need exactly one law. Controls may appear in expressions but never receive a state-transition law. Parameters share the identifier namespace with variables, so a collision fails construction.

`WorldConfig` enables symbol and unit validation. Symbol validation checks every identifier read by a law. Unit validation compares the inferred expression dimension with the target state unit. Configure units before constructing the world if dimensional validity matters to the run.

`DiscreteWorld` has the same invariants and evaluates simultaneous updates through the discrete simulator. It does not represent a mixed discrete/continuous model.
