# Shocks and scenarios

A deterministic shock can be represented as a scheduled input or parameter override. Define its onset, duration (including a return assignment where appropriate), magnitude, unit, and rationale in a scenario record.

```sh
lawsynth simulate model.lsworld --initial x=1 --start 0 --end 6 --step 0.01 \
  --input-at 2.0:forcing=4 --input-at 3.0:forcing=0 > shock.csv
```

Compare a shocked run with an otherwise identical baseline using the same integration horizon and output sampling. Avoid interpreting deterministic differences as a probability of impact.

Stochastic forcing, random shocks, distributional ensembles, and uncertainty propagation are not implemented by this CLI. Generate any external ensemble deliberately and retain its random seed and sampling method.
