# Scheduled interventions

Apply a value from a simulation time onward with `--parameter-at TIME:NAME=VALUE` or `--input-at TIME:NAME=VALUE`. Multiple assignments are accepted and are carried in the simulation request as scheduled parameter or input changes.

```sh
lawsynth simulate model.lsworld --initial x=1 --start 0 --end 12 --step 0.01 \
  --parameter-at 4.0:growth=0.1 --input-at 8.0:forcing=0 > intervention.csv
```

Schedule times and values must be finite. Record the intended physical event and units outside the compact assignment syntax. If several assignments occur at a time, retain their original scenario file or command so they can be audited.

Interventions are exogenous value overrides. They do not establish causality, infer counterfactuals, trigger state-dependent guards, or model treatment compliance.
