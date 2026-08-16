# Simulating a world bundle

Simulation executes a validated continuous or discrete world bundle produced by LawSynth. Inspect the bundle first, supply every required initial state, choose a finite horizon and positive step, and save the emitted CSV as an analysis artifact.

```sh
lawsynth inspect model.lsworld
lawsynth simulate model.lsworld --initial x=1 --initial y=0 \
  --start 0 --end 20 --step 0.01 > run.csv
```

The continuous command uses `--start`, `--end`, and `--step`; discrete worlds use `simulate-discrete` and `--steps`. The output is numeric CSV headed by `time` followed by state identifiers. Simulated trajectories are consequences of the fitted model, not observations or uncertainty intervals.

Use [initial state](initial-state.md), [interventions](interventions.md), and [export](export.md) to keep each scenario explicit and reproducible.
