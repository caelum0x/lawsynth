# Initial state

Supply one `--initial NAME=VALUE` assignment for each continuous world state. Values must be finite and identifiers must match the saved bundle exactly. Inspect the bundle before scripting simulations so a renamed or omitted state fails before a costly run.

```sh
lawsynth simulate model.lsworld --initial prey=12 --initial predator=3 \
  --start 0 --end 30 --step 0.05 > trajectory.csv
```

An initial state is a scenario input, not a parameter estimate. Preserve its source—measured, assumed, or sampled—in the run record. For boundary-sensitive systems, test nearby initial values and report whether the conclusion changes.

The CLI does not infer missing states, draw initial conditions from a distribution, or optimise them against target observations. Perform those procedures in an explicit external analysis loop.
