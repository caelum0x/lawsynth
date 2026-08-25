# Information diffusion analysis

`lawsynth-information-diffusion` is a native Rust application for bounded,
reproducible analysis of measured network cascades. It estimates a global
independent-cascade transmission probability from observed exposures, evaluates
it on a chronological holdout when the data permits, and compares baseline and
intervention forecasts under the same calibrated model.

The canonical implementation lives in `apps/information-diffusion`. Hackathon
directories contain submission material only.

## Data contract

The application reads six normalized UTF-8 TSV relations. Five are required:

- nodes: `node_id`;
- directed edges: `source`, `target`;
- cascades: `cascade_id`, canonical UTC `started_at`, and
  `observation_end_step`;
- activations: `cascade_id`, `node_id`, and discrete `step`; and
- forecast seeds: `node_id`.

An optional `node_id` relation defines blocked nodes for the intervention.
Headers and column counts are exact. Identifiers, references, uniqueness,
observation windows, timestamps, and configured resource bounds are validated
before analysis. Inputs larger than 64 MiB per file are rejected at the CLI
boundary.

## Analysis and receipts

For each inactive node-step with at least one newly active parent, calibration
records the one-time incoming edge exposure count and whether the node activates
at the next step. This matches the synchronous independent-cascade forecast,
where an edge gets one attempt when its source activates. A bounded
maximum-likelihood fit estimates the global edge probability. The report keeps
its confidence interval, negative log likelihood, and Brier score separate from
the seeded Monte Carlo forecast bands.

Cascades are sorted by start time and identifier. With at least three cascades,
the first 80 percent calibrate a holdout model and the remaining cascades measure
out-of-sample Brier score and log loss. If either partition has no measurable
edge exposures, the backtest is `unmeasured` with an explicit reason.

The baseline and blocked-node/transmission-multiplier intervention use identical
seed and simulation settings. Every report includes schema and model versions,
normalized data digest, model inputs, deterministic simulation seed, calibrated
metrics, forecasts, limitations, and a SHA-256 receipt digest. Output is synced
and installed atomically; replacement requires explicit `--overwrite`.

See [`apps/information-diffusion/README.md`](../../apps/information-diffusion/README.md)
for exact headers and CLI flags.

## Interpretation boundary

An edge means activation timing is compatible with a candidate path. It does
not establish causal influence; shared causes and missing nodes can produce the
same pattern. Absence is interpreted only inside the declared observation
window. Process bands do not include parameter uncertainty, which remains a
separate calibration interval.
