# Discrete logistic map

This deterministic P2 benchmark generates observations for `x[t+1] = 3.7*x[t]*(1-x[t])`.

## Capability contract

LawSynth can simulate a serialized DiscreteWorld, but its public CLI currently has no discrete discovery/model-authoring command. This benchmark therefore cannot produce a native candidate without fabricating a bundle.

Run `python generate.py --workdir /tmp/discrete` to materialize the CSV. Run
`python run.py --workdir /tmp/discrete` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
