# Feature operators

The supported library is configured from a polynomial degree plus optional trigonometric and rational features. Use `--degree N`, `--trigonometric`, and `--rational` only when those functional forms are plausible and observable over the data range.

```sh
lawsynth discover data.csv --time time --state x,y --degree 3 \
  --trigonometric --solver sr3 --threshold 0.02 --output model.lsworld
```

More operators create more ways to fit noise. Inspect the number of terms and validate trajectories outside the fitting window before expanding the library. A rational term near a zero denominator requires particular scientific scrutiny even when numerical fitting succeeds.

Custom operators, plugin loading, neural priors, and arbitrary user expressions are not exposed by the production CLI. Do not represent a preprocessing transform as an engine-supported operator unless it is captured as part of the input provenance.
