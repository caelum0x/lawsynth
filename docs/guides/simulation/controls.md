# Inputs and parameter overrides

Set constant values with `--parameter NAME=VALUE` and `--input NAME=VALUE`. These are named overrides evaluated by the world; they must be finite and use bundle identifiers.

```sh
lawsynth simulate model.lsworld --initial x=1 --start 0 --end 10 --step 0.01 \
  --parameter growth=0.3 --input forcing=1.0 > controlled.csv
```

Keep a scenario manifest containing every override and its unit. A parameter override changes the model assumption; it should not be conflated with a measured external input. Validate override names against the inspected bundle and reject a run if an intended control is unused.

There is no closed-loop controller, parameter optimiser, or automatic control-policy search in the CLI. Those require an external controller that calls simulation under an explicit safety review.
