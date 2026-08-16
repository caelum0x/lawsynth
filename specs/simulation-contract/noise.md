# Noise

The deterministic simulate and simulate_discrete entry points do not add process,
observation, or parameter noise. Their output is deterministic for an identical
validated world and request.

lawsynth-sim includes an explicit Euler--Maruyama helper and SdeConfig for local
SDE calculations, but it is not integrated into SimulationRequest, World IR
compilation, trajectory artifacts, or the CLI. It MUST NOT be represented as a
supported stochastic-world simulation feature. Seeds and uncertainty envelopes
are likewise outside this contract.
