# Trajectories

Trajectory contains time: Vec<f64> and values: BTreeMap<Identifier, Vec<f64>>.
Each variable vector has exactly one value per timestamp; keys are state
identifiers in deterministic map order. The first row is the initial state and
every emitted number is finite.

The contract carries no units, confidence bands, events, derivatives, metadata,
or serialization format. Those must be attached by a higher-level artifact
contract. Consumers MUST preserve the time vector rather than assuming a fixed
number of samples or an equal interval after scheduled changes.
