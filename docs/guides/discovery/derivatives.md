# Derivative estimation

Discovery estimates derivatives from observations, so derivative settings materially affect inferred equations. The CLI offers a finite baseline and one selectable alternative: `--savgol-window ODD_N`, `--spline`, `--spectral`, or `--tvreg-lambda VALUE` with optional `--tvreg-iterations`. It rejects selecting more than one alternative.

```sh
lawsynth discover data.csv --time time --state x \
  --savgol-window 9 --output model.lsworld
```

Validate the smoothing choice on withheld data. Smooth estimators can suppress measurement noise and also remove genuine fast dynamics. Spectral methods are especially sensitive to sampling assumptions and boundaries.

No derivative choice repairs unordered time, missing values, or an experiment with inadequate temporal resolution. Compare at least a baseline and one justified alternative, then retain both the configuration and validation evidence.
