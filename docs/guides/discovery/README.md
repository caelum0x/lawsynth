# Discovering a continuous world

Discovery estimates explicit continuous-time equations from an ordered numeric dataset. The operational path is: validate observations, choose state columns, fit a deliberately bounded feature library, inspect the resulting bundle, then simulate it on data not used for fitting.

```sh
lawsynth discover observations.csv --time time --state x,y \
  --degree 2 --solver stlsq --threshold 0.05 --output model.lsworld
lawsynth inspect model.lsworld
lawsynth simulate model.lsworld --initial x=1 --initial y=0 \
  --start 0 --end 10 --step 0.01 > forecast.csv
```

The first candidate is a sparse regression result, not proof of a causal or unique mechanism. Compare candidates under documented preprocessing and hold out whole time intervals or experiments. The current interface models continuous state dynamics; it does not claim automatic regime detection, latent-state inference, causal identification, or uncertainty-calibrated model selection.
