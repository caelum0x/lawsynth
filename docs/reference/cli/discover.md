# `lawsynth discover`

```
lawsynth discover observations.csv --time t --state x,y --output model.lsworld
```

The input is a comma-delimited numeric CSV with a header. The time column must exist, contain finite strictly increasing values, and every remaining selected numeric field must be finite and aligned. `--state` is a nonempty comma-separated list of state column identifiers. Discovery writes a continuous v0.1 bundle and reports the leading candidate's MSE and expression complexity.

Supported options are `--degree N`, `--threshold VALUE`, `--solver stlsq|sr3`, `--trigonometric`, `--rational`, `--smooth-radius N`, `--bootstrap REPLICATES`, and `--symbolic-depth N`. Choose at most one derivative option: `--savgol-window ODD_N`, `--spline`, `--spectral`, or `--tvreg-lambda VALUE` (with optional `--tvreg-iterations N`). Values must be finite where numeric. Missing data, quoted/escaped CSV dialects, categorical data, causal discovery, and remote execution are not CLI features.
